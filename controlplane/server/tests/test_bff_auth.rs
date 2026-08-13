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
use frn_core::identity::{SessionKey, SessionPayload, User};
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
        tokio::spawn(async move {
            axum::serve(listener, bff.into_router()).await.ok();
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

    /// Registers the IdP token endpoint returning an error status (refresh
    /// rejection).
    fn stub_token_endpoint_error(&mut self, status: usize) {
        let token_mock = self
            .idp
            .server
            .mock("POST", "/oauth/token")
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":"invalid_grant"}"#)
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
            "given_name": "Wile",
            "family_name": "Coyote",
        });
        if let Some(nonce) = nonce {
            claims["nonce"] = json!(nonce);
        }
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

    assert_eq!(response.status().as_u16(), 400);
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

    assert_eq!(response.status().as_u16(), 400);
    assert!(set_cookie(&response, "frn_session").is_none());
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

    assert_eq!(response.status().as_u16(), 401);
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
    // token.
    let refreshed_id = harness.id_token("wile.coyote@acme.org", None, now() + 7200);
    harness.stub_token_endpoint_with(&refreshed_id, "rt-rotated");

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
}

#[tokio::test]
async fn refresh_clears_the_cookie_when_the_idp_rejects() {
    let mut harness = Harness::start(lazy_pool()).await;
    let cookie = harness.seal_session("rt-stale", "wile.coyote@acme.org", now() + 5);

    // The IdP rejects the refresh grant.
    harness.stub_token_endpoint_error(400);

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
}
