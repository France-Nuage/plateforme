//! Black-box HTTP tests for the confidential-client BFF (`/auth/*`).
//!
//! These drive the BFF over a real TCP socket with `reqwest`, against a real
//! (mockito) OpenID provider — no mocks of our own code. They cover the security
//! contract: `state` (CSRF) and `nonce` (replay) validation, the server-side
//! code→token exchange, the id_token checks, `/auth/me` shape (including the
//! authoritative admin flag), and RP-initiated logout.
//!
//! The flows that do not read the database build the BFF with a lazily-connected
//! pool. The cases that resolve `users.is_admin` (`/auth/me`, and the callback →
//! me round-trip) are `#[sqlx::test]` and require a database.

use std::time::{SystemTime, UNIX_EPOCH};

use auth::OpenID;
use auth::mock::WithJwks;
use fabrique::Factory;
use frn_core::identity::{SESSION_COOKIE_NAME, SessionKey, SessionPayload, User};
use mock_server::MockServer;
use serde_json::{Value, json};
use server::bff::{Bff, SameSite, Settings};
use sqlx::PgPool;
use uuid::Uuid;

const CLIENT_ID: &str = "francenuage-bff";
const CLIENT_SECRET: &str = "test-confidential-secret";
const REDIRECT_URL: &str = "https://controlplane.test/auth/callback";
const CONSOLE_URL: &str = "https://console.test";
/// Deterministic key the harness seals/opens session cookies with (also passed
/// to the BFF so both agree). Not a secret — test-only.
const COOKIE_KEY: [u8; 32] = [31u8; 32];
const SESSION_MAX_AGE_SECS: i64 = 12 * 3600;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

/// A live BFF plus the mock IdP backing it.
struct Harness {
    base: String,
    idp: MockServer,
    idp_base: String,
    client: reqwest::Client,
    /// Same key the BFF seals with — lets tests forge/inspect session cookies.
    cookie_key: SessionKey,
}

impl Harness {
    /// Boots a mock IdP (well-known + JWKS) and a BFF wired to it, serving on an
    /// ephemeral port.
    async fn start(pool: PgPool) -> Self {
        let mut idp = MockServer::new().await.with_jwks();
        let idp_base = idp.url();

        let well_known = json!({
            "issuer": idp_base,
            "authorization_endpoint": format!("{idp_base}/oauth/authorize"),
            "token_endpoint": format!("{idp_base}/oauth/token"),
            "end_session_endpoint": format!("{idp_base}/oauth/logout"),
            "jwks_uri": format!("{idp_base}/oauth/discovery/keys"),
        })
        .to_string();
        let well_known_mock = idp
            .server
            .mock("GET", "/.well-known/openid-configuration")
            .with_body(well_known)
            .create();
        idp.mocks.push(well_known_mock);

        let openid = OpenID::discover(
            reqwest::Client::new(),
            &format!("{idp_base}/.well-known/openid-configuration"),
        )
        .await
        .expect("could not discover mock openid");

        let cookie_key = SessionKey::from_bytes(COOKIE_KEY);
        let settings = Settings {
            oidc_url: format!("{idp_base}/.well-known/openid-configuration"),
            client_id: CLIENT_ID.to_owned(),
            client_secret: CLIENT_SECRET.to_owned(),
            redirect_url: REDIRECT_URL.to_owned(),
            console_url: CONSOLE_URL.to_owned(),
            cookie_domain: None,
            cookie_secure: true,
            cookie_same_site: SameSite::Lax,
            cookie_key: cookie_key.clone(),
            session_max_age_secs: SESSION_MAX_AGE_SECS,
        };

        let bff = Bff::discover(openid, pool, settings)
            .await
            .expect("could not build bff");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("could not bind bff listener");
        let addr = listener.local_addr().expect("could not read local addr");
        // Mount `/metrics` on the same origin as `/auth/*`, mirroring production
        // (`application.rs`), so tests can scrape the auth counters end-to-end.
        let router = bff
            .into_router()
            .route("/metrics", axum::routing::get(render_metrics));
        tokio::spawn(async move {
            axum::serve(listener, router).await.ok();
        });

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("could not build http client");

        Self {
            base: format!("http://{addr}"),
            idp,
            idp_base,
            client,
            cookie_key,
        }
    }

    /// Registers the IdP token endpoint returning the given id_token plus a
    /// refresh token (so the BFF can seal a refreshable session).
    fn stub_token_endpoint(&mut self, id_token: &str) {
        self.stub_token_endpoint_with(id_token, "refresh-token-from-idp");
    }

    /// Registers the IdP token endpoint returning `id_token` + `refresh_token`.
    fn stub_token_endpoint_with(&mut self, id_token: &str, refresh_token: &str) {
        let body = json!({
            "access_token": "opaque-access-token",
            "id_token": id_token,
            "refresh_token": refresh_token,
            "token_type": "Bearer",
            "expires_in": 3600,
        })
        .to_string();
        let token_mock = self
            .idp
            .server
            .mock("POST", "/oauth/token")
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.idp.mocks.push(token_mock);
    }

    /// Registers the IdP token endpoint returning a 200 that carries NO id_token,
    /// so the callback hits the `no_id_token` reject.
    fn stub_token_endpoint_without_id_token(&mut self) {
        let body = json!({
            "access_token": "opaque-access-token",
            "refresh_token": "refresh-token-from-idp",
            "token_type": "Bearer",
            "expires_in": 3600,
        })
        .to_string();
        let token_mock = self
            .idp
            .server
            .mock("POST", "/oauth/token")
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.idp.mocks.push(token_mock);
    }

    /// Seals a session cookie value exactly as the BFF would.
    fn seal_session(&self, refresh_token: &str, email: &str, exp: u64) -> String {
        self.cookie_key
            .seal(&SessionPayload {
                refresh_token: refresh_token.to_owned(),
                sub: "subject-abc-123".to_owned(),
                email: email.to_owned(),
                exp,
            })
            .expect("could not seal session cookie")
    }

    /// Opens a session cookie value the BFF produced.
    fn open_session(&self, cookie: &str) -> SessionPayload {
        self.cookie_key
            .open(cookie)
            .expect("could not open session cookie")
    }

    /// Mints an id_token signed with the mock key.
    fn id_token(&self, email: &str, nonce: Option<&str>, exp: u64) -> String {
        let mut claims = json!({
            "iss": self.idp_base,
            "aud": CLIENT_ID,
            "exp": exp,
            "iat": now(),
            "sub": "subject-abc-123",
            "email": email,
            "email_verified": true,
            "given_name": "Wile",
            "family_name": "Coyote",
        });
        if let Some(nonce) = nonce {
            claims["nonce"] = json!(nonce);
        }
        OpenID::sign_claims(&claims)
    }

    /// Mints a signed id_token whose email is present but NOT verified.
    fn id_token_unverified_email(&self, email: &str, nonce: &str, exp: u64) -> String {
        let claims = json!({
            "iss": self.idp_base,
            "aud": CLIENT_ID,
            "exp": exp,
            "iat": now(),
            "sub": "subject-abc-123",
            "email": email,
            "email_verified": false,
            "nonce": nonce,
        });
        OpenID::sign_claims(&claims)
    }

    /// Mints a signed id_token with a verified email but NO `sub` claim.
    fn id_token_without_sub(&self, email: &str, nonce: &str, exp: u64) -> String {
        let claims = json!({
            "iss": self.idp_base,
            "aud": CLIENT_ID,
            "exp": exp,
            "iat": now(),
            "email": email,
            "email_verified": true,
            "nonce": nonce,
        });
        OpenID::sign_claims(&claims)
    }
}

/// A pool that never connects — sufficient for the flows that do not query the DB.
fn lazy_pool() -> PgPool {
    PgPool::connect_lazy("postgres://bff-tests@localhost/none").expect("could not build lazy pool")
}

/// Extracts a `Set-Cookie` value (up to the first `;`) for `name`.
fn set_cookie(response: &reqwest::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|cookie| {
            let (pair, _) = cookie.split_once(';').unwrap_or((cookie, ""));
            let (cookie_name, cookie_value) = pair.split_once('=')?;
            (cookie_name.trim() == name).then(|| cookie_value.trim().to_owned())
        })
}

/// Whether a `Set-Cookie` header for `name` carries the given attribute.
fn cookie_has_attribute(response: &reqwest::Response, name: &str, attribute: &str) -> bool {
    response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter(|cookie| cookie.starts_with(&format!("{name}=")))
        .any(|cookie| {
            cookie
                .split(';')
                .any(|part| part.trim().eq_ignore_ascii_case(attribute))
        })
}

#[tokio::test]
async fn login_redirects_to_authorization_endpoint_with_state_and_nonce() {
    let harness = Harness::start(lazy_pool()).await;

    let response = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login request failed");

    assert_eq!(response.status().as_u16(), 302);

    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("login must redirect")
        .to_owned();

    assert!(location.starts_with(&format!("{}/oauth/authorize", harness.idp_base)));
    assert!(location.contains("response_type=code"));
    assert!(location.contains(&format!("client_id={CLIENT_ID}")));
    assert!(location.contains("state="));
    assert!(location.contains("nonce="));

    // The CSRF/replay tokens are stashed as httpOnly cookies.
    assert!(set_cookie(&response, "frn_oauth_state").is_some());
    assert!(set_cookie(&response, "frn_oauth_nonce").is_some());
    assert!(cookie_has_attribute(
        &response,
        "frn_oauth_state",
        "HttpOnly"
    ));
}

#[tokio::test]
async fn callback_rejects_a_state_mismatch() {
    let harness = Harness::start(lazy_pool()).await;

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=whatever&state=forged",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            "frn_oauth_state=legitimate; frn_oauth_nonce=n0nce",
        )
        .send()
        .await
        .expect("callback request failed");

    // A rejected callback now 302-redirects to the console origin with a
    // machine-readable `?auth_error=<reason>` (instead of a bare text page on the
    // control-plane origin), and never establishes a session.
    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(
        location.starts_with(CONSOLE_URL),
        "must redirect to the console origin, got {location}"
    );
    assert!(
        location.contains("auth_error=state"),
        "must carry the state reject reason, got {location}"
    );
    assert!(
        set_cookie(&response, "frn_session").is_none(),
        "a rejected callback must never establish a session"
    );
}

#[tokio::test]
async fn callback_rejects_a_missing_state() {
    let harness = Harness::start(lazy_pool()).await;

    let response = harness
        .client
        .get(format!("{}/auth/callback?code=whatever", harness.base))
        .header(reqwest::header::COOKIE, "frn_oauth_state=legitimate")
        .send()
        .await
        .expect("callback request failed");

    // A missing state is a `state` reject: 302 to the console with `?auth_error=state`.
    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(location.contains("auth_error=state"));
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn callback_rejects_a_provider_error() {
    let harness = Harness::start(lazy_pool()).await;

    // The IdP redirected back with `?error=...` (e.g. the user denied consent):
    // an `exchange` reject, checked before anything else.
    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?error=access_denied&state=whatever",
            harness.base
        ))
        .send()
        .await
        .expect("callback request failed");

    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(
        location.contains("auth_error=exchange"),
        "must carry the exchange reject reason, got {location}"
    );
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn callback_rejects_a_missing_nonce_cookie() {
    let harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");

    // State matches but the nonce cookie is absent → `nonce` reject (before the
    // code exchange, so no token endpoint is even contacted).
    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(reqwest::header::COOKIE, format!("frn_oauth_state={state}"))
        .send()
        .await
        .expect("callback request failed");

    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(
        location.contains("auth_error=nonce"),
        "must carry the nonce reject reason, got {location}"
    );
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn callback_rejects_a_token_response_without_id_token() {
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    // The token endpoint answers 200 but omits the id_token → `no_id_token` reject.
    harness.stub_token_endpoint_without_id_token();

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback request failed");

    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(
        location.contains("auth_error=no_id_token"),
        "must carry the no_id_token reject reason, got {location}"
    );
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn callback_rejects_an_unverified_email() {
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    // The IdP returns a validly-signed id_token (iss/aud/exp/nonce all pass) but
    // `email_verified` is false: an attacker could have self-registered a
    // victim's (e.g. an admin's) address without proving mailbox control. Since
    // identity is keyed on the email, this MUST be rejected with no session.
    let id_token =
        harness.id_token_unverified_email("admin@francenuage.fr", &nonce, now() + 3600);
    harness.stub_token_endpoint(&id_token);

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback request failed");

    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(
        location.contains("auth_error=validation"),
        "an unverified email must be an id_token validation reject, got {location}"
    );
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn callback_rejects_a_missing_sub() {
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    // The IdP returns a validly-signed id_token (iss/aud/exp/nonce/email_verified
    // all pass) but carries NO `sub`. The subject is pinned to the user row so a
    // recycled email cannot inherit a former owner's identity, so a token without
    // a stable subject cannot mint a session and MUST be rejected with none.
    let id_token = harness.id_token_without_sub("admin@francenuage.fr", &nonce, now() + 3600);
    harness.stub_token_endpoint(&id_token);

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback request failed");

    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(
        location.contains("auth_error=validation"),
        "a missing sub must be an id_token validation reject, got {location}"
    );
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn me_reports_unauthenticated_for_an_expired_session() {
    let harness = Harness::start(lazy_pool()).await;

    // A well-formed sealed cookie whose inner `exp` has already elapsed must fail
    // closed on `/auth/me` — the anonymous shape, never the identity. This gate
    // is a distinct code path from the gRPC-path `exp` check in frn-core, so a
    // regression removing it would not be caught by the gRPC-path tests.
    let expired = harness.seal_session("rt", "wile.coyote@acme.org", now() - 1);

    let body: Value = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={expired}"))
        .send()
        .await
        .expect("me request failed")
        .json()
        .await
        .expect("me must return json");

    assert_eq!(body["authenticated"], json!(false));
}

#[sqlx::test(migrations = "../migrations")]
async fn callback_completes_the_flow_and_sets_an_encrypted_session(pool: PgPool) {
    let mut harness = Harness::start(pool).await;

    // 1. Login to obtain a matching state/nonce pair.
    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    // 2. The IdP returns an id_token bound to that nonce.
    let id_token = harness.id_token("wile.coyote@acme.org", Some(&nonce), now() + 3600);
    harness.stub_token_endpoint(&id_token);

    // 3. Complete the callback with the browser's login cookies.
    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback failed");

    assert_eq!(response.status().as_u16(), 302);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some(CONSOLE_URL)
    );

    let session = set_cookie(&response, "frn_session").expect("session cookie must be set");
    // The cookie is now an opaque, encrypted blob — never the raw id_token.
    assert_ne!(
        session, id_token,
        "the session cookie must not carry the id_token"
    );
    assert!(cookie_has_attribute(&response, "frn_session", "HttpOnly"));
    assert!(cookie_has_attribute(&response, "frn_session", "Secure"));

    // The sealed cookie authenticates a follow-up `/auth/me` call.
    let body: Value = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={session}"))
        .send()
        .await
        .expect("me request failed")
        .json()
        .await
        .expect("me must return json");

    assert_eq!(body["authenticated"], json!(true));
    assert_eq!(body["email"], json!("wile.coyote@acme.org"));
}

#[tokio::test]
async fn callback_rejects_a_nonce_mismatch() {
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    // id_token minted with a different nonce than the one the browser holds.
    let id_token = harness.id_token("wile.coyote@acme.org", Some("tampered-nonce"), now() + 3600);
    harness.stub_token_endpoint(&id_token);

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback failed");

    // The nonce mismatch is caught during id_token validation, so it surfaces as
    // a `validation` reject: 302 to the console with `?auth_error=validation`.
    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a rejected callback must redirect to the console");
    assert!(location.starts_with(CONSOLE_URL));
    assert!(location.contains("auth_error=validation"));
    assert!(set_cookie(&response, "frn_session").is_none());
}

#[tokio::test]
async fn me_reports_unauthenticated_without_a_session() {
    let harness = Harness::start(lazy_pool()).await;

    let body: Value = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .send()
        .await
        .expect("me request failed")
        .json()
        .await
        .expect("me must return json");

    assert_eq!(body["authenticated"], json!(false));
}

#[tokio::test]
async fn logout_clears_the_session_and_redirects_to_end_session() {
    let harness = Harness::start(lazy_pool()).await;
    let session = harness.seal_session("rt", "wile.coyote@acme.org", now() + 3600);

    let response = harness
        .client
        .get(format!("{}/auth/logout", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={session}"))
        .send()
        .await
        .expect("logout failed");

    assert_eq!(response.status().as_u16(), 302);

    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("logout must redirect")
        .to_owned();
    assert!(location.starts_with(&format!("{}/oauth/logout", harness.idp_base)));
    assert!(location.contains("post_logout_redirect_uri="));
    assert!(location.contains(&format!("client_id={CLIENT_ID}")));
    // The sealed session no longer carries the id_token, so no `id_token_hint`.
    assert!(!location.contains("id_token_hint="));

    // The session cookie is expired (Max-Age=0).
    assert_eq!(set_cookie(&response, "frn_session").as_deref(), Some(""));
    assert!(cookie_has_attribute(&response, "frn_session", "Max-Age=0"));
}

#[sqlx::test(migrations = "../migrations")]
async fn me_reports_admin_from_the_database(pool: PgPool) {
    User::factory()
        .id(Uuid::new_v4())
        .email("admin@francenuage.fr".to_owned())
        // Unpinned (NULL) subject so the first /auth/me pins the cookie's subject;
        // the factory's Faker would otherwise fill `sub` with a random value that
        // never matches the session cookie → SubjectMismatch (flaky rejection).
        .sub(None)
        .is_admin(true)
        .create(&pool)
        .await
        .expect("could not seed admin user");

    let harness = Harness::start(pool).await;
    let session = harness.seal_session("rt", "admin@francenuage.fr", now() + 3600);

    let body: Value = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={session}"))
        .send()
        .await
        .expect("me request failed")
        .json()
        .await
        .expect("me must return json");

    assert_eq!(body["authenticated"], json!(true));
    assert_eq!(body["email"], json!("admin@francenuage.fr"));
    assert_eq!(body["isAdmin"], json!(true));
}

#[sqlx::test(migrations = "../migrations")]
async fn me_pins_a_pre_existing_null_subject_row(pool: PgPool) {
    // A row that predates subject-pinning has `sub = NULL` — exactly the state of
    // every existing user right after the migration adds the column. Its owner's
    // first login must pin the cookie's subject and authenticate normally, never
    // lock the user out.
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, is_admin) VALUES ($1, $2, true)")
        .bind(id)
        .bind("legacy-admin@francenuage.fr")
        .execute(&pool)
        .await
        .expect("could not seed a pre-pinning admin row");

    let harness = Harness::start(pool.clone()).await;
    let session = harness.seal_session("rt", "legacy-admin@francenuage.fr", now() + 3600);

    let body: Value = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={session}"))
        .send()
        .await
        .expect("me request failed")
        .json()
        .await
        .expect("me must return json");

    assert_eq!(body["authenticated"], json!(true));
    assert_eq!(body["isAdmin"], json!(true));

    // The subject was pinned to the row on that first login.
    let pinned: (Option<String>,) = sqlx::query_as("SELECT sub FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("could not read back the pinned subject");
    assert_eq!(pinned.0.as_deref(), Some("subject-abc-123"));
}

#[sqlx::test(migrations = "../migrations")]
async fn me_reports_non_admin_for_a_fresh_user(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let session = harness.seal_session("rt", "regular@francenuage.fr", now() + 3600);

    let body: Value = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={session}"))
        .send()
        .await
        .expect("me request failed")
        .json()
        .await
        .expect("me must return json");

    assert_eq!(body["authenticated"], json!(true));
    assert_eq!(body["isAdmin"], json!(false));
}

#[tokio::test]
async fn refresh_rotates_the_session_on_success() {
    let mut harness = Harness::start(lazy_pool()).await;

    // A session whose access is about to expire, carrying a refresh token.
    let cookie = harness.seal_session("rt-initial", "wile.coyote@acme.org", now() + 5);

    // The IdP hands back a fresh id_token (longer expiry) and a rotated refresh
    // token — and must be hit AT MOST ONCE, proving refresh is bounded
    // server-side (no retry storm on a single `/auth/refresh`).
    let refreshed_id = harness.id_token("wile.coyote@acme.org", None, now() + 7200);
    let token_body = json!({
        "access_token": "opaque-access-token",
        "id_token": refreshed_id,
        "refresh_token": "rt-rotated",
        "token_type": "Bearer",
        "expires_in": 3600,
    })
    .to_string();
    let token_mock = harness
        .idp
        .server
        .mock("POST", "/oauth/token")
        .with_header("content-type", "application/json")
        .with_body(token_body)
        .expect(1)
        .create();

    let response = harness
        .client
        .get(format!("{}/auth/refresh", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={cookie}"))
        .send()
        .await
        .expect("refresh failed");

    assert_eq!(response.status().as_u16(), 200);

    let new_cookie = set_cookie(&response, "frn_session").expect("a fresh session cookie");
    assert_ne!(new_cookie, cookie, "the cookie must be resealed");
    assert!(cookie_has_attribute(&response, "frn_session", "HttpOnly"));

    // The new inner exp is the refreshed token's, and the refresh token rotated.
    let payload = harness.open_session(&new_cookie);
    assert_eq!(payload.exp, now() + 7200);
    assert_eq!(payload.refresh_token, "rt-rotated");
    assert_eq!(payload.email, "wile.coyote@acme.org");

    // The IdP token endpoint was hit exactly once (bounded refresh).
    token_mock.assert_async().await;
}

#[tokio::test]
async fn refresh_clears_the_cookie_when_the_idp_rejects() {
    let mut harness = Harness::start(lazy_pool()).await;
    // Force the process-global Prometheus recorder to install BEFORE the drive
    // below: `metrics::counter!` is a no-op until the first render, so an
    // increment emitted before any scrape would be silently lost.
    warm_up_metrics();
    let cookie = harness.seal_session("rt-stale", "wile.coyote@acme.org", now() + 5);

    // The IdP rejects the refresh grant — hit AT MOST ONCE (bounded, no retry).
    let token_mock = harness
        .idp
        .server
        .mock("POST", "/oauth/token")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"invalid_grant"}"#)
        .expect(1)
        .create();

    let response = harness
        .client
        .get(format!("{}/auth/refresh", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={cookie}"))
        .send()
        .await
        .expect("refresh failed");

    // Fails closed: 401 + the session cookie cleared (Max-Age=0), never a 500.
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(set_cookie(&response, "frn_session").as_deref(), Some(""));
    assert!(cookie_has_attribute(&response, "frn_session", "Max-Age=0"));

    // The IdP token endpoint was hit exactly once (bounded refresh).
    token_mock.assert_async().await;

    // A cookie WAS presented and the IdP rejected the grant => this is a genuine
    // refresh failure, labelled `rejected` (the series the failure-ratio alert
    // targets), never `no_session`.
    let metrics = harness
        .client
        .get(format!("{}/metrics", harness.base))
        .send()
        .await
        .expect("metrics request failed")
        .text()
        .await
        .expect("metrics body");
    assert!(
        counter_value(&metrics, r#"auth_refresh_total{result="rejected"}"#) >= 1,
        "a cookie-present IdP-reject must count as result=\"rejected\":\n{metrics}"
    );
}

#[tokio::test]
async fn refresh_rejects_an_unverified_email() {
    let mut harness = Harness::start(lazy_pool()).await;
    // Install the recorder before the drive so the `rejected` counter is recorded
    // (no-op until the first render).
    warm_up_metrics();

    // A live session about to expire, carrying a refresh token.
    let cookie = harness.seal_session("rt-initial", "wile.coyote@acme.org", now() + 5);

    // The IdP answers the refresh grant with a validly-signed id_token whose email
    // is present but NOT verified. Refresh re-validates the fresh id_token exactly
    // like the callback (via `validate_id_token`), so it must fail closed here too:
    // a session must never be resealed from an unverified email on renewal — the
    // same self-registered-address → victim-row vector the callback rejects, on the
    // refresh path. Hit AT MOST ONCE (bounded refresh, no retry storm).
    let refreshed_id = harness.id_token_unverified_email(
        "admin@francenuage.fr",
        "unused-on-refresh",
        now() + 7200,
    );
    let token_body = json!({
        "access_token": "opaque-access-token",
        "id_token": refreshed_id,
        "refresh_token": "rt-rotated",
        "token_type": "Bearer",
        "expires_in": 3600,
    })
    .to_string();
    let token_mock = harness
        .idp
        .server
        .mock("POST", "/oauth/token")
        .with_header("content-type", "application/json")
        .with_body(token_body)
        .expect(1)
        .create();

    let response = harness
        .client
        .get(format!("{}/auth/refresh", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={cookie}"))
        .send()
        .await
        .expect("refresh failed");

    // Fails closed: 401 + the session cookie cleared (Max-Age=0), never a resealed
    // session and never a 500.
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(set_cookie(&response, "frn_session").as_deref(), Some(""));
    assert!(cookie_has_attribute(&response, "frn_session", "Max-Age=0"));

    // The IdP token endpoint was hit exactly once (bounded refresh).
    token_mock.assert_async().await;

    // A cookie WAS presented and the refreshed id_token failed validation => a
    // genuine refresh failure, labelled `rejected` (the series the failure-ratio
    // alert targets), never `no_session`.
    let metrics = harness
        .client
        .get(format!("{}/metrics", harness.base))
        .send()
        .await
        .expect("metrics request failed")
        .text()
        .await
        .expect("metrics body");
    assert!(
        counter_value(&metrics, r#"auth_refresh_total{result="rejected"}"#) >= 1,
        "an unverified-email refresh must count as result=\"rejected\":\n{metrics}"
    );
}

#[tokio::test]
async fn refresh_rejects_a_missing_sub() {
    let mut harness = Harness::start(lazy_pool()).await;
    // Install the recorder before the drive so the `rejected` counter is recorded.
    warm_up_metrics();

    // A live session about to expire, carrying a refresh token.
    let cookie = harness.seal_session("rt-initial", "wile.coyote@acme.org", now() + 5);

    // The IdP answers the refresh grant with a validly-signed id_token that carries
    // a verified email but NO `sub`. Refresh re-validates the fresh id_token like
    // the callback, so a token with no stable subject must fail closed here too:
    // the subject is pinned to the user row and a session is never resealed without
    // one. Hit AT MOST ONCE (bounded refresh, no retry storm).
    let refreshed_id =
        harness.id_token_without_sub("admin@francenuage.fr", "unused-on-refresh", now() + 7200);
    let token_body = json!({
        "access_token": "opaque-access-token",
        "id_token": refreshed_id,
        "refresh_token": "rt-rotated",
        "token_type": "Bearer",
        "expires_in": 3600,
    })
    .to_string();
    let token_mock = harness
        .idp
        .server
        .mock("POST", "/oauth/token")
        .with_header("content-type", "application/json")
        .with_body(token_body)
        .expect(1)
        .create();

    let response = harness
        .client
        .get(format!("{}/auth/refresh", harness.base))
        .header(reqwest::header::COOKIE, format!("frn_session={cookie}"))
        .send()
        .await
        .expect("refresh failed");

    // Fails closed: 401 + the session cookie cleared (Max-Age=0), never a resealed
    // session and never a 500.
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(set_cookie(&response, "frn_session").as_deref(), Some(""));
    assert!(cookie_has_attribute(&response, "frn_session", "Max-Age=0"));

    // The IdP token endpoint was hit exactly once (bounded refresh).
    token_mock.assert_async().await;

    // A cookie WAS presented and the refreshed id_token failed validation => a
    // genuine refresh failure, labelled `rejected`, never `no_session`.
    let metrics = harness
        .client
        .get(format!("{}/metrics", harness.base))
        .send()
        .await
        .expect("metrics request failed")
        .text()
        .await
        .expect("metrics body");
    assert!(
        counter_value(&metrics, r#"auth_refresh_total{result="rejected"}"#) >= 1,
        "a missing-sub refresh must count as result=\"rejected\":\n{metrics}"
    );
}

#[tokio::test]
async fn refresh_rejects_a_tampered_session_cookie() {
    let harness = Harness::start(lazy_pool()).await;
    // Install the recorder before the drive so the decrypt_fail counter is
    // recorded (no-op until the first render).
    warm_up_metrics();

    // An undecryptable session cookie must fail closed: 401 + the cookie cleared
    // (Max-Age=0), never a 500 — and it never even reaches the IdP.
    let response = harness
        .client
        .get(format!("{}/auth/refresh", harness.base))
        .header(reqwest::header::COOKIE, "frn_session=not-a-sealed-token")
        .send()
        .await
        .expect("refresh request failed");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(set_cookie(&response, "frn_session").as_deref(), Some(""));
    assert!(cookie_has_attribute(&response, "frn_session", "Max-Age=0"));

    // An unopenable cookie is a `decrypt_fail` refresh outcome — a numerator term
    // of the refresh-failure-ratio alert (an AUTH_COOKIE_KEY-rotation eviction it
    // must catch), so prove the exact series is emitted end-to-end.
    let metrics = harness
        .client
        .get(format!("{}/metrics", harness.base))
        .send()
        .await
        .expect("metrics request failed")
        .text()
        .await
        .expect("metrics body");
    assert!(
        counter_value(&metrics, r#"auth_refresh_total{result="decrypt_fail"}"#) >= 1,
        "an undecryptable cookie must count as result=\"decrypt_fail\":\n{metrics}"
    );
}

#[tokio::test]
async fn me_reports_unauthenticated_for_a_tampered_session_cookie() {
    let harness = Harness::start(lazy_pool()).await;

    // A tampered/undecryptable cookie yields the anonymous shape (HTTP 200,
    // authenticated:false), never a 500.
    let response = harness
        .client
        .get(format!("{}/auth/me", harness.base))
        .header(reqwest::header::COOKIE, "frn_session=not-a-sealed-token")
        .send()
        .await
        .expect("me request failed");

    assert_eq!(response.status().as_u16(), 200);
    let body: Value = response.json().await.expect("me must return json");
    assert_eq!(body["authenticated"], json!(false));
}

#[tokio::test]
async fn callback_seals_a_session_cookie_under_the_browser_size_limit() {
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    let id_token = harness.id_token("wile.coyote@acme.org", Some(&nonce), now() + 3600);
    harness.stub_token_endpoint(&id_token);

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback failed");

    assert_eq!(response.status().as_u16(), 302);
    let session = set_cookie(&response, "frn_session").expect("session cookie must be set");
    assert!(
        session.len() < 4096,
        "a normal sealed session must stay under the 4 KB browser cookie limit, got {} bytes",
        session.len()
    );
}

#[tokio::test]
async fn callback_rejects_an_oversized_refresh_token_without_setting_a_cookie() {
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");

    // An IdP refresh token so large the sealed cookie would blow past the 4 KB
    // browser limit. The BFF must fail loud, never emit an oversized cookie the
    // browser would silently drop (which would be a broken session).
    let oversized_refresh_token = "a".repeat(6000);
    let id_token = harness.id_token("wile.coyote@acme.org", Some(&nonce), now() + 3600);
    harness.stub_token_endpoint_with(&id_token, &oversized_refresh_token);

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback failed");

    // Loud, machine-readable failure back to the console origin — and crucially
    // NO oversized session cookie handed to the browser.
    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("callback must redirect");
    assert!(
        location.starts_with(CONSOLE_URL),
        "must redirect to the console origin, got {location}"
    );
    assert!(
        location.contains("auth_error=session"),
        "a server-side sizing failure is a `session` reject, not `validation`, got {location}"
    );
    assert!(
        set_cookie(&response, "frn_session").is_none(),
        "an oversized session must never be shipped to the browser"
    );
}

#[test]
fn console_cors_origin_normalizes_a_trailing_slash_to_the_bare_origin() {
    // A CONSOLE_URL with a trailing slash (or a path/query) must still yield the
    // bare scheme+host origin the browser sends in `Origin`; otherwise credentialed
    // CORS silently blocks every call.
    let bare = server::config::console_cors_origin("https://console.france-nuage.fr")
        .expect("valid absolute https origin");
    assert_eq!(
        bare.to_str().expect("header-safe origin"),
        "https://console.france-nuage.fr"
    );

    let trailing_slash = server::config::console_cors_origin("https://console.france-nuage.fr/")
        .expect("valid absolute https origin");
    assert_eq!(
        trailing_slash, bare,
        "a trailing slash must not change the CORS origin"
    );

    let with_path =
        server::config::console_cors_origin("https://console.france-nuage.fr/login?next=%2Fx")
            .expect("valid absolute https origin");
    assert_eq!(with_path, bare, "a path/query must not change the CORS origin");

    // A non-default port is preserved (matches the browser `Origin`).
    let ported = server::config::console_cors_origin("http://localhost:5173/")
        .expect("valid absolute http origin");
    assert_eq!(
        ported.to_str().expect("header-safe origin"),
        "http://localhost:5173"
    );

    // Not an absolute URL => fail loud at startup, never a silently-broken origin.
    assert!(server::config::console_cors_origin("not-a-url").is_err());
    assert!(server::config::console_cors_origin("/relative/path").is_err());
}

#[test]
fn grpc_web_transport_headers_are_within_the_cors_allow_list() {
    // The headers a gRPC-web browser transport puts on every unary call (grpc-web
    // protocol): the request content-type, the grpc-web marker, the client
    // user-agent, and the optional deadline. If any is absent from the BFF CORS
    // allow-list, the browser's preflight fails and the call never runs — so this
    // guards against a future allow-list edit silently breaking preflight.
    let transport_headers = ["content-type", "x-grpc-web", "x-user-agent", "grpc-timeout"];
    let allow_list: std::collections::HashSet<String> = server::config::bff_cors_allow_headers()
        .into_iter()
        .map(|header| header.as_str().to_owned())
        .collect();

    for header in transport_headers {
        assert!(
            allow_list.contains(header),
            "gRPC-web sends `{header}` but the BFF CORS allow-list omits it — preflight would fail"
        );
    }
}

#[test]
fn cross_site_lax_cookie_policy_is_flagged_at_startup() {
    // Console and control plane on different registrable domains + SameSite=Lax:
    // the browser withholds the cookie on cross-site subresource calls → warn.
    assert!(
        server::config::same_site_cross_site_warning(
            "https://console.example.com",
            "https://api.controlplane.io/auth/callback",
            SameSite::Lax,
            true,
        )
        .is_some()
    );

    // Same registrable domain (only the subdomain differs): Lax is safe → no warning.
    assert!(
        server::config::same_site_cross_site_warning(
            "https://console.france-nuage.fr",
            "https://api.france-nuage.fr/auth/callback",
            SameSite::Lax,
            true,
        )
        .is_none()
    );

    // Cross-site but `SameSite=None; Secure` — the only cross-site-safe policy.
    assert!(
        server::config::same_site_cross_site_warning(
            "https://console.example.com",
            "https://api.controlplane.io/auth/callback",
            SameSite::None,
            true,
        )
        .is_none()
    );

    // Cross-site + `SameSite=None` but NOT Secure: browsers reject None without
    // Secure, so it stays unsafe → warn.
    assert!(
        server::config::same_site_cross_site_warning(
            "https://console.example.com",
            "https://api.controlplane.io/auth/callback",
            SameSite::None,
            false,
        )
        .is_some()
    );
}

#[test]
fn cross_site_under_a_multi_label_public_suffix_is_flagged() {
    // `app.co.uk` and `api.co.uk` are DIFFERENT registrable domains, but the
    // last-two-labels heuristic collapses both to `co.uk` and would call them
    // same-site — suppressing the warning exactly when it is needed. The safer
    // default biases toward emitting it rather than silently under-warning.
    let warning = server::config::same_site_cross_site_warning(
        "https://app.co.uk",
        "https://api.co.uk/auth/callback",
        SameSite::Lax,
        true,
    )
    .expect("a public-suffix-shaped shared site must warn");
    // The message must not contradict itself: both sites collapse to `co.uk`, so
    // it must NOT claim "different registrable domains (co.uk vs co.uk)".
    assert!(
        !warning.contains("different registrable domains"),
        "same-site public-suffix warning must not claim 'different': {warning}"
    );
    assert!(warning.contains("co.uk"), "warning should name the shared site: {warning}");

    // Sanity: a genuine same-registrable-domain pair (subdomains only, non
    // public-suffix shape) is still correctly suppressed — no spurious warning.
    assert!(
        server::config::same_site_cross_site_warning(
            "https://console.france-nuage.fr",
            "https://api.france-nuage.fr/auth/callback",
            SameSite::Lax,
            true,
        )
        .is_none()
    );
}

#[test]
fn console_cors_origin_rejects_non_http_schemes() {
    // A tuple origin is not enough: ws/wss/ftp also yield tuple origins, but they
    // never equal the browser's https `Origin` — a `CONSOLE_URL=ws://…` typo would
    // silently block every credentialed call. Fail loud at startup instead.
    assert!(server::config::console_cors_origin("ws://console.france-nuage.fr").is_err());
    assert!(server::config::console_cors_origin("wss://console.france-nuage.fr").is_err());
    assert!(server::config::console_cors_origin("ftp://console.france-nuage.fr").is_err());

    // http(s) still pass.
    assert!(server::config::console_cors_origin("https://console.france-nuage.fr").is_ok());
    assert!(server::config::console_cors_origin("http://localhost:5173").is_ok());
}

#[test]
fn session_max_ttl_honors_a_valid_non_default_value() {
    // A valid override (positive integer seconds) must be honored verbatim, never
    // silently replaced by the default.
    assert_eq!(server::config::parse_session_max_ttl(Some("86400")), 86400);
    // Unset falls back to the documented default.
    assert_eq!(
        server::config::parse_session_max_ttl(None),
        server::bff::DEFAULT_SESSION_MAX_TTL_SECS
    );
}

#[test]
#[should_panic(expected = "SESSION_MAX_TTL")]
fn session_max_ttl_fails_loud_on_an_unparseable_value() {
    // `"12h"` is not a valid i64 — the old code silently fell back to the 12h
    // default, ignoring the operator. It must now fail loud at startup.
    let _ = server::config::parse_session_max_ttl(Some("12h"));
}

#[tokio::test]
async fn callback_rejects_a_session_cookie_that_overflows_only_with_the_name_prefix() {
    // The browser bounds the whole `frn_session=` + value PAIR at ~4 KB, not the
    // value alone. This drives a sealed value that fits the 4096 B value-only view
    // yet overflows once the name prefix is added — the ~11-byte window the
    // value-only bound missed (which the browser silently drops → login loop).
    let mut harness = Harness::start(lazy_pool()).await;

    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");
    let exp = now() + 3600;

    // Find a refresh-token length whose sealed cookie value lands in the boundary
    // window: value ≤ limit (old bound passes) but name + "=" + value > limit (new
    // bound fails). Sealing is deterministic in the input length, so measuring via
    // the harness matches exactly what the callback will produce.
    const LIMIT: usize = 4096;
    let overhead = SESSION_COOKIE_NAME.len() + 1; // name + "="
    let boundary_refresh_token = (2500..LIMIT)
        .map(|len| "a".repeat(len))
        .find(|token| {
            let sealed = harness.seal_session(token, "wile.coyote@acme.org", exp);
            sealed.len() <= LIMIT && overhead + sealed.len() > LIMIT
        })
        .expect("a refresh token whose sealed value lands in the name-prefix boundary window");

    let id_token = harness.id_token("wile.coyote@acme.org", Some(&nonce), exp);
    harness.stub_token_endpoint_with(&id_token, &boundary_refresh_token);

    let response = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback failed");

    // The value alone would have passed the old bound, but the pair overflows =>
    // fail loud (no cookie), redirect to the console with `?auth_error=session`.
    assert_eq!(response.status().as_u16(), 302);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("callback must redirect");
    assert!(
        location.starts_with(CONSOLE_URL),
        "must redirect to the console origin, got {location}"
    );
    assert!(
        location.contains("auth_error=session"),
        "an oversized session is a `session` reject, got {location}"
    );
    assert!(
        set_cookie(&response, "frn_session").is_none(),
        "a cookie that would overflow the browser pair limit must never be shipped"
    );
}

#[tokio::test]
async fn metrics_endpoint_reflects_auth_counters() {
    // O1-A guard: install the recorder, drive one `/auth/callback` reject and one
    // `/auth/refresh` outcome, then scrape `/metrics` on the same origin and assert
    // the exact series the Grafana alert keys on are present and non-zero. Without
    // the recorder installed first the `counter!` calls are no-ops, so this also
    // proves the metric wiring is live end-to-end.
    server::metrics::handle();

    let mut harness = Harness::start(lazy_pool()).await;

    // (1) A callback that fails to seal (oversized refresh token) => a `session`
    // reject, distinct from an id_token `validation` failure.
    let login = harness
        .client
        .get(format!("{}/auth/login", harness.base))
        .send()
        .await
        .expect("login failed");
    let state = set_cookie(&login, "frn_oauth_state").expect("state cookie");
    let nonce = set_cookie(&login, "frn_oauth_nonce").expect("nonce cookie");
    let id_token = harness.id_token("wile.coyote@acme.org", Some(&nonce), now() + 3600);
    harness.stub_token_endpoint_with(&id_token, &"a".repeat(6000));
    let callback = harness
        .client
        .get(format!(
            "{}/auth/callback?code=auth-code&state={state}",
            harness.base
        ))
        .header(
            reqwest::header::COOKIE,
            format!("frn_oauth_state={state}; frn_oauth_nonce={nonce}"),
        )
        .send()
        .await
        .expect("callback failed");
    assert!(
        callback
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .expect("callback redirect")
            .contains("auth_error=session")
    );

    // (2) A refresh with no session cookie => a `no_session` outcome (benign
    // anonymous probe), NOT a `rejected` failure — kept out of the alert.
    let refresh = harness
        .client
        .get(format!("{}/auth/refresh", harness.base))
        .send()
        .await
        .expect("refresh failed");
    assert_eq!(refresh.status().as_u16(), 401);

    // (3) Scrape /metrics and assert the exact series are present and non-zero.
    let metrics = harness
        .client
        .get(format!("{}/metrics", harness.base))
        .send()
        .await
        .expect("metrics request failed")
        .text()
        .await
        .expect("metrics body");

    assert!(
        counter_value(&metrics, r#"auth_callback_reject_total{reason="session"}"#) >= 1,
        "auth_callback_reject_total{{reason=\"session\"}} must be present and non-zero:\n{metrics}"
    );
    assert!(
        counter_value(&metrics, r#"auth_refresh_total{result="no_session"}"#) >= 1,
        "a cookieless /auth/refresh must count as result=\"no_session\":\n{metrics}"
    );
}

/// Renders the process-global Prometheus metrics; mounted on the harness at the
/// same origin as `/auth/*` (mirrors production `application.rs`).
async fn render_metrics() -> String {
    server::metrics::render()
}

/// Installs the process-global Prometheus recorder. `metrics::counter!` is a
/// no-op until the recorder is installed (on the first render), so any test that
/// asserts a counter value must call this BEFORE driving the flow it measures —
/// otherwise the increment is silently dropped when this test happens to run
/// before any `/metrics` scrape elsewhere in the process.
fn warm_up_metrics() {
    let _ = server::metrics::render();
}

/// Extracts the integer value of a Prometheus counter line by its exact
/// `name{labels}` prefix; `0` when the series is absent.
fn counter_value(rendered: &str, series: &str) -> i64 {
    rendered
        .lines()
        .find_map(|line| line.strip_prefix(series)?.trim().parse::<f64>().ok())
        .map(|value| value as i64)
        .unwrap_or(0)
}
