//! Backend-For-Frontend (BFF) confidential-client OIDC flow.
//!
//! France Nuage is migrating its IAM to FerrisKey, which does **not** support
//! PKCE on its OAuth RP surface. A public SPA can therefore no longer protect
//! itself with PKCE, so the browser console becomes a **confidential client**:
//! the authorization-code exchange moves server-side into the control plane
//! (which holds `client_id` + `client_secret`), and the browser receives an
//! **httpOnly session cookie** instead of tokens in JavaScript.
//!
//! The flow is provider-agnostic — it works against the current Keycloak IdP and
//! against FerrisKey — which makes it the safe migration path. It is guarded by
//! configuration (see [`crate::config::Config`]): the routes only exist when an
//! `OIDC_CLIENT_SECRET` is provided, so the legacy SPA/PKCE flow keeps working
//! untouched until the confidential client is provisioned on the IdP.
//!
//! ## Endpoints (all mounted on the control-plane origin, next to gRPC-web)
//!
//! - `GET /auth/login` — mint `state` + `nonce`, stash them in short-lived
//!   httpOnly cookies, redirect to the IdP `authorization_endpoint`.
//! - `GET /auth/callback` — validate `state` (CSRF), exchange `code` → tokens
//!   server-side, validate the id_token (`iss`, `aud`, `exp`, `nonce`), seal an
//!   encrypted session cookie, redirect back to the console.
//! - `GET /auth/refresh` — swap the stored refresh token for fresh tokens at the
//!   IdP and reseal the cookie; fails closed (clear cookie + 401) on any error.
//! - `GET /auth/me` — return the current session identity + authoritative
//!   `isAdmin` flag for the SPA.
//! - `GET|POST /auth/logout` — clear the session cookie and redirect to the IdP
//!   `end_session_endpoint` (RP-initiated logout).
//!
//! The session cookie carries an **encrypted, self-contained** session
//! ([`frn_core::identity::SessionPayload`], sealed with
//! [`frn_core::identity::SessionKey`]): a refresh token plus the short-lived
//! identity, never the fat id_token and never anything in cleartext. Browser
//! gRPC-web calls send it automatically and the control plane authenticates them
//! by opening it in [`frn_core::identity::IAM::principal`]. No token is ever
//! exposed to JavaScript.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use frn_core::identity::{SESSION_COOKIE_NAME, SessionKey, SessionPayload, User};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use auth::OpenID;

use crate::metrics::{self, CallbackReject, RefreshResult};

/// Cookie holding the OAuth `state` (CSRF token) across the login round-trip.
const STATE_COOKIE: &str = "frn_oauth_state";
/// Cookie holding the OIDC `nonce` (replay token) across the login round-trip.
const NONCE_COOKIE: &str = "frn_oauth_nonce";
/// Upper bound on the login round-trip (state/nonce cookies TTL), in seconds.
const OAUTH_COOKIE_TTL_SECS: i64 = 600;
/// Default session cookie `Max-Age` (the **refresh window**): how long the
/// browser retains the cookie so it can call `/auth/refresh`. This is
/// independent of — and longer than — the sealed payload's inner `exp` (the
/// short access lifetime). Overridable via `SESSION_MAX_TTL`.
pub const DEFAULT_SESSION_MAX_TTL_SECS: i64 = 12 * 3600;
/// Bound on every outbound HTTP call to the IdP (discovery + token exchange +
/// refresh).
const HTTP_TIMEOUT_SECS: u64 = 10;

/// `SameSite` attribute for the BFF cookies.
#[derive(Clone, Copy, Debug)]
pub enum SameSite {
    Lax,
    Strict,
    None,
}

impl SameSite {
    fn as_str(self) -> &'static str {
        match self {
            SameSite::Lax => "Lax",
            SameSite::Strict => "Strict",
            SameSite::None => "None",
        }
    }

    /// Parses the `AUTH_COOKIE_SAMESITE` env value, defaulting to `Lax`.
    pub fn from_env_value(value: Option<&str>) -> Self {
        match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("strict") => SameSite::Strict,
            Some("none") => SameSite::None,
            _ => SameSite::Lax,
        }
    }
}

/// Cookie shaping shared by every cookie the BFF sets.
#[derive(Clone, Debug)]
struct CookieConfig {
    domain: Option<String>,
    secure: bool,
    same_site: SameSite,
}

impl CookieConfig {
    /// Builds a `Set-Cookie` value for `name`/`value` with a bounded `max_age`.
    fn set(&self, name: &str, value: &str, max_age: i64) -> String {
        let mut cookie = format!(
            "{name}={value}; HttpOnly; Path=/; SameSite={}; Max-Age={max_age}",
            self.same_site.as_str()
        );
        if self.secure {
            cookie.push_str("; Secure");
        }
        if let Some(domain) = &self.domain {
            cookie.push_str("; Domain=");
            cookie.push_str(domain);
        }
        cookie
    }

    /// Builds a `Set-Cookie` value that immediately expires `name`.
    fn clear(&self, name: &str) -> String {
        self.set(name, "", 0)
    }
}

/// Errors raised while driving the confidential-client flow.
///
/// Variants never carry the `client_secret` or any token material, so they are
/// safe to log.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oidc discovery failed for {0}")]
    Discovery(String),

    #[error("http client build failed: {0}")]
    HttpClient(String),

    #[error("token endpoint unreachable")]
    TokenEndpointUnreachable,

    #[error("token exchange rejected with status {0}")]
    TokenExchangeRejected(u16),

    #[error("unparsable token response")]
    UnparsableTokenResponse,

    #[error("identity provider returned no id_token")]
    MissingIdToken,

    #[error("malformed id token")]
    MalformedIdToken,

    #[error("invalid id token signature")]
    InvalidSignature,

    #[error("id token issuer mismatch")]
    IssuerMismatch,

    #[error("id token audience mismatch")]
    AudienceMismatch,

    #[error("id token expired")]
    Expired,

    #[error("id token nonce mismatch")]
    NonceMismatch,

    #[error("id token missing email claim")]
    MissingEmailClaim,

    #[error("session cookie has no refresh token")]
    MissingRefreshToken,

    #[error("refresh token rejected with status {0}")]
    RefreshRejected(u16),

    #[error("could not seal the session cookie")]
    SessionSeal,
}

/// Settings required to build a [`Bff`], read from the environment at startup.
#[derive(Clone)]
pub struct Settings {
    /// OIDC discovery URL (`.../.well-known/openid-configuration`).
    pub oidc_url: String,
    /// Confidential client id registered on the IdP.
    pub client_id: String,
    /// Confidential client secret (kept out of `Debug` output below).
    pub client_secret: String,
    /// Registered redirect URI (the absolute URL of `/auth/callback`).
    pub redirect_url: String,
    /// Where to land the browser after login/logout (the console origin).
    pub console_url: String,
    /// Optional cookie `Domain` (e.g. `.france-nuage.fr` to share across
    /// console/controlplane subdomains).
    pub cookie_domain: Option<String>,
    /// Whether to mark cookies `Secure` (always true outside local http dev).
    pub cookie_secure: bool,
    /// `SameSite` attribute for the cookies.
    pub cookie_same_site: SameSite,
    /// Server key that seals/opens the encrypted session cookie (`AUTH_COOKIE_KEY`).
    pub cookie_key: SessionKey,
    /// Session cookie `Max-Age` in seconds (the refresh window). The inner
    /// payload `exp` (short access lifetime) is independent and comes from the
    /// id_token.
    pub session_max_age_secs: i64,
}

impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("oidc_url", &self.oidc_url)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[redacted]")
            .field("redirect_url", &self.redirect_url)
            .field("console_url", &self.console_url)
            .field("cookie_domain", &self.cookie_domain)
            .field("cookie_secure", &self.cookie_secure)
            .field("cookie_same_site", &self.cookie_same_site)
            .field("cookie_key", &self.cookie_key)
            .field("session_max_age_secs", &self.session_max_age_secs)
            .finish()
    }
}

/// OIDC provider endpoints the BFF needs (a superset of what
/// [`auth::OpenID`] keeps, which only covers signature validation).
#[derive(Clone, Debug, Deserialize)]
struct ProviderMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    end_session_endpoint: Option<String>,
}

/// Confidential-client OIDC backend-for-frontend.
///
/// Cheaply cloneable: it is stored in the server [`crate::config::Config`] and
/// mounted as an axum sub-router.
#[derive(Clone)]
pub struct Bff {
    openid: OpenID,
    db: Pool<Postgres>,
    http: reqwest::Client,
    metadata: ProviderMetadata,
    client_id: String,
    client_secret: String,
    redirect_url: String,
    console_url: String,
    cookie: CookieConfig,
    /// Seals/opens the encrypted session cookie. Same key as the gRPC cookie
    /// path in `IAM` (both from `AUTH_COOKIE_KEY`).
    session_key: SessionKey,
    /// Session cookie `Max-Age` (refresh window), in seconds.
    session_max_age_secs: i64,
}

impl Bff {
    /// Discovers the IdP endpoints and builds a ready-to-serve BFF.
    ///
    /// `openid` is reused for id_token signature validation so the JWK cache is
    /// shared with the gRPC auth path.
    pub async fn discover(
        openid: OpenID,
        db: Pool<Postgres>,
        settings: Settings,
    ) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .build()
            .map_err(|err| Error::HttpClient(err.to_string()))?;

        let metadata: ProviderMetadata = http
            .get(&settings.oidc_url)
            .send()
            .await
            .map_err(|_| Error::Discovery(settings.oidc_url.clone()))?
            .json()
            .await
            .map_err(|_| Error::Discovery(settings.oidc_url.clone()))?;

        Ok(Self {
            openid,
            db,
            http,
            metadata,
            client_id: settings.client_id,
            client_secret: settings.client_secret,
            redirect_url: settings.redirect_url,
            console_url: settings.console_url,
            cookie: CookieConfig {
                domain: settings.cookie_domain,
                secure: settings.cookie_secure,
                same_site: settings.cookie_same_site,
            },
            session_key: settings.cookie_key,
            session_max_age_secs: settings.session_max_age_secs,
        })
    }

    /// Builds the axum sub-router exposing the `/auth/*` endpoints.
    pub fn into_router(self) -> Router {
        Router::new()
            .route("/auth/login", get(login))
            .route("/auth/callback", get(callback))
            .route("/auth/refresh", get(refresh))
            .route("/auth/me", get(me))
            .route("/auth/logout", get(logout).post(logout))
            .with_state(self)
    }

    /// Exchanges an authorization `code` for tokens as a confidential client
    /// (`client_secret_post`).
    async fn exchange_code(&self, code: &str) -> Result<TokenResponse, Error> {
        let response = self
            .http
            .post(&self.metadata.token_endpoint)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", self.redirect_url.as_str()),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await
            .map_err(|_| Error::TokenEndpointUnreachable)?;

        if !response.status().is_success() {
            return Err(Error::TokenExchangeRejected(response.status().as_u16()));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|_| Error::UnparsableTokenResponse)
    }

    /// Exchanges a `refresh_token` for a fresh token set (`client_secret_post`).
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenResponse, Error> {
        let response = self
            .http
            .post(&self.metadata.token_endpoint)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await
            .map_err(|_| Error::TokenEndpointUnreachable)?;

        if !response.status().is_success() {
            return Err(Error::RefreshRejected(response.status().as_u16()));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|_| Error::UnparsableTokenResponse)
    }

    /// Seals a [`SessionPayload`] from validated id_token `claims`, carrying
    /// `refresh_token` for the next `/auth/refresh`. The inner `exp` is the
    /// token's own (short) expiry; the cookie `Max-Age` (refresh window) is set
    /// by the caller.
    fn seal_session(&self, claims: &IdTokenClaims, refresh_token: String) -> Result<String, Error> {
        let email = claims.email.clone().ok_or(Error::MissingEmailClaim)?;
        let payload = SessionPayload {
            refresh_token,
            sub: claims.sub.clone().unwrap_or_default(),
            email,
            exp: claims.exp,
        };
        self.session_key
            .seal(&payload)
            .map_err(|_| Error::SessionSeal)
    }

    /// Validates an id_token: signature (via the shared JWK cache), then
    /// `iss`, `aud`, `exp`, and — when `expected_nonce` is provided — `nonce`.
    async fn validate_id_token(
        &self,
        token: &str,
        expected_nonce: Option<&str>,
    ) -> Result<IdTokenClaims, Error> {
        // 1. Cryptographic signature. This is the crypto-critical gate; the
        //    payload can only be trusted after it passes.
        self.openid
            .validate_token(token)
            .await
            .map_err(|_| Error::InvalidSignature)?;

        // 2. Claims. The payload is now signature-verified, so decoding it is safe.
        let claims = decode_id_token_claims(token)?;

        if claims.iss != self.metadata.issuer {
            return Err(Error::IssuerMismatch);
        }
        if !aud_contains(&claims.aud, &self.client_id) {
            return Err(Error::AudienceMismatch);
        }
        if claims.exp <= unix_now() {
            return Err(Error::Expired);
        }
        if let Some(expected) = expected_nonce {
            match &claims.nonce {
                Some(got) if constant_time_eq(got, expected) => {}
                _ => return Err(Error::NonceMismatch),
            }
        }

        Ok(claims)
    }
}

/// `GET /auth/login` — start the authorization-code flow.
async fn login(State(bff): State<Bff>) -> Response {
    let state = Uuid::new_v4().simple().to_string();
    let nonce = Uuid::new_v4().simple().to_string();

    let mut authorize_url = match reqwest::Url::parse(&bff.metadata.authorization_endpoint) {
        Ok(url) => url,
        Err(_) => {
            tracing::error!("invalid authorization_endpoint in provider metadata");
            return reject(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid authorization endpoint",
                &[],
            );
        }
    };
    authorize_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &bff.client_id)
        .append_pair("redirect_uri", &bff.redirect_url)
        // `offline_access` requests a refresh token so the BFF can renew the
        // session server-side via `/auth/refresh`.
        .append_pair("scope", "openid profile email offline_access")
        .append_pair("state", &state)
        .append_pair("nonce", &nonce);

    let cookies = [
        bff.cookie.set(STATE_COOKIE, &state, OAUTH_COOKIE_TTL_SECS),
        bff.cookie.set(NONCE_COOKIE, &nonce, OAUTH_COOKIE_TTL_SECS),
    ];
    metrics::login();
    redirect(authorize_url.as_str(), &cookies)
}

/// Query parameters the IdP appends to the callback redirect.
#[derive(Debug, Deserialize)]
struct CallbackParams {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// `GET /auth/callback` — finish the flow server-side.
async fn callback(
    State(bff): State<Bff>,
    headers: HeaderMap,
    Query(params): Query<CallbackParams>,
) -> Response {
    // Whatever happens, the one-shot login cookies are cleared.
    let clear = [
        bff.cookie.clear(STATE_COOKIE),
        bff.cookie.clear(NONCE_COOKIE),
    ];

    if let Some(error) = params.error {
        tracing::warn!(error = %error, "oidc provider returned an error at callback");
        metrics::callback_reject(CallbackReject::Exchange);
        return reject(
            StatusCode::BAD_REQUEST,
            "authentication failed at the identity provider",
            &clear,
        );
    }

    // CSRF: the state must be present on both sides and match exactly.
    let expected_state = cookie_value(&headers, STATE_COOKIE);
    match (&params.state, &expected_state) {
        (Some(got), Some(want)) if constant_time_eq(got, want) => {}
        _ => {
            tracing::warn!("oidc callback rejected: state mismatch");
            metrics::callback_reject(CallbackReject::State);
            return reject(StatusCode::BAD_REQUEST, "invalid state", &clear);
        }
    }

    let nonce = match cookie_value(&headers, NONCE_COOKIE) {
        Some(nonce) => nonce,
        None => {
            tracing::warn!("oidc callback rejected: missing nonce cookie");
            metrics::callback_reject(CallbackReject::Nonce);
            return reject(StatusCode::BAD_REQUEST, "missing nonce", &clear);
        }
    };

    let code = match params.code {
        Some(code) => code,
        None => {
            metrics::callback_reject(CallbackReject::Exchange);
            return reject(StatusCode::BAD_REQUEST, "missing authorization code", &clear);
        }
    };

    let tokens = match bff.exchange_code(&code).await {
        Ok(tokens) => tokens,
        Err(err) => {
            tracing::warn!(error = %err, "authorization code exchange failed");
            metrics::callback_reject(CallbackReject::Exchange);
            return reject(StatusCode::BAD_GATEWAY, "token exchange failed", &clear);
        }
    };

    let id_token = match tokens.id_token {
        Some(id_token) => id_token,
        None => {
            tracing::warn!("token response carried no id_token");
            metrics::callback_reject(CallbackReject::NoIdToken);
            return reject(StatusCode::BAD_GATEWAY, "no id token", &clear);
        }
    };

    let claims = match bff.validate_id_token(&id_token, Some(&nonce)).await {
        Ok(claims) => claims,
        Err(err) => {
            tracing::warn!(error = %err, "id_token validation failed");
            metrics::callback_reject(CallbackReject::Validation);
            return reject(StatusCode::UNAUTHORIZED, "invalid id token", &clear);
        }
    };

    // Seal an encrypted session: the refresh token (for `/auth/refresh`) plus the
    // short-lived identity. The cookie `Max-Age` is the long refresh window; the
    // sealed payload's inner `exp` is the token's own short expiry.
    let sealed = match bff.seal_session(&claims, tokens.refresh_token.unwrap_or_default()) {
        Ok(sealed) => sealed,
        Err(err) => {
            tracing::warn!(error = %err, "could not seal the session cookie");
            metrics::callback_reject(CallbackReject::Validation);
            return reject(StatusCode::UNAUTHORIZED, "invalid id token", &clear);
        }
    };

    let cookies = [
        bff.cookie.clear(STATE_COOKIE),
        bff.cookie.clear(NONCE_COOKIE),
        bff.cookie
            .set(SESSION_COOKIE_NAME, &sealed, bff.session_max_age_secs),
    ];
    redirect(&bff.console_url, &cookies)
}

/// `GET /auth/refresh` — renew the session server-side.
///
/// Opens the sealed cookie, swaps its refresh token for fresh tokens at the IdP,
/// re-validates the id_token (no nonce on refresh), and reseals a cookie with a
/// new inner `exp` and the rotated refresh token. Every failure path — no
/// cookie, decrypt failure, missing/rejected refresh token, invalid id_token —
/// **fails closed**: the cookie is cleared and the response is 401, never a 500
/// and never a silent success.
async fn refresh(State(bff): State<Bff>, headers: HeaderMap) -> Response {
    let cleared = [bff.cookie.clear(SESSION_COOKIE_NAME)];

    let cookie = match cookie_value(&headers, SESSION_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => {
            metrics::refresh(RefreshResult::Rejected);
            return reject(StatusCode::UNAUTHORIZED, "no session", &cleared);
        }
    };

    let payload = match bff.session_key.open(&cookie) {
        Ok(payload) => payload,
        Err(_) => {
            tracing::warn!("auth refresh rejected: session cookie could not be opened");
            metrics::refresh(RefreshResult::DecryptFail);
            return reject(StatusCode::UNAUTHORIZED, "invalid session", &cleared);
        }
    };

    if payload.refresh_token.is_empty() {
        tracing::warn!("auth refresh rejected: session carries no refresh token");
        metrics::refresh(RefreshResult::Rejected);
        return reject(StatusCode::UNAUTHORIZED, "no refresh token", &cleared);
    }

    let tokens = match bff.refresh_tokens(&payload.refresh_token).await {
        Ok(tokens) => tokens,
        Err(err) => {
            tracing::warn!(error = %err, "auth refresh rejected by identity provider");
            metrics::refresh(RefreshResult::Rejected);
            return reject(StatusCode::UNAUTHORIZED, "refresh rejected", &cleared);
        }
    };

    let id_token = match &tokens.id_token {
        Some(id_token) => id_token,
        None => {
            tracing::warn!("auth refresh rejected: refreshed response carried no id_token");
            metrics::refresh(RefreshResult::Rejected);
            return reject(StatusCode::UNAUTHORIZED, "refresh rejected", &cleared);
        }
    };

    let claims = match bff.validate_id_token(id_token, None).await {
        Ok(claims) => claims,
        Err(err) => {
            tracing::warn!(error = %err, "auth refresh rejected: id_token validation failed");
            metrics::refresh(RefreshResult::Rejected);
            return reject(StatusCode::UNAUTHORIZED, "refresh rejected", &cleared);
        }
    };

    // Rotate the refresh token when the IdP issued a new one; otherwise keep the
    // existing one (some providers do not rotate on every refresh).
    let refresh_token = tokens
        .refresh_token
        .filter(|token| !token.is_empty())
        .unwrap_or(payload.refresh_token);

    let sealed = match bff.seal_session(&claims, refresh_token) {
        Ok(sealed) => sealed,
        Err(err) => {
            tracing::warn!(error = %err, "auth refresh rejected: could not reseal session");
            metrics::refresh(RefreshResult::Rejected);
            return reject(StatusCode::UNAUTHORIZED, "refresh rejected", &cleared);
        }
    };

    metrics::refresh(RefreshResult::Ok);
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        header_value(&bff.cookie.set(SESSION_COOKIE_NAME, &sealed, bff.session_max_age_secs)),
    );
    (StatusCode::OK, headers, Json(RefreshResponse { refreshed: true })).into_response()
}

/// `GET /auth/refresh` success body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshResponse {
    refreshed: bool,
}

/// Identity payload returned to the SPA by `GET /auth/me`.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeResponse {
    authenticated: bool,
    sub: String,
    email: String,
    first_name: String,
    last_name: String,
    picture: String,
    is_admin: bool,
}

/// `GET /auth/me` — current identity + authoritative admin flag for the SPA.
///
/// Opens the sealed session cookie and enforces its short inner `exp`. A missing,
/// unopenable, or expired cookie returns the anonymous `authenticated: false`
/// shape (the SPA then triggers `/auth/refresh` or `/auth/login`).
///
/// The minimal session payload carries no profile claims, so `firstName` /
/// `lastName` / `picture` are returned empty; the fields stay in the JSON so the
/// contract is stable. (Sourcing display fields from the DB is a Phase-2 choice.)
async fn me(State(bff): State<Bff>, headers: HeaderMap) -> Response {
    let cookie = match cookie_value(&headers, SESSION_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return Json(MeResponse::default()).into_response(),
    };

    let payload = match bff.session_key.open(&cookie) {
        Ok(payload) => payload,
        Err(_) => return Json(MeResponse::default()).into_response(),
    };

    if payload.exp <= unix_now() {
        return Json(MeResponse::default()).into_response();
    }

    // The admin flag is authoritative from the control-plane database (the exact
    // same source as the Profile.GetCurrentUser RPC), never from a token role.
    let is_admin = match User::find_or_create_one_by_email(&bff.db, &payload.email).await {
        Ok(user) => user.is_admin,
        Err(err) => {
            tracing::error!(error = %err, "failed to resolve user for /auth/me");
            return reject(
                StatusCode::INTERNAL_SERVER_ERROR,
                "could not resolve identity",
                &[],
            );
        }
    };

    Json(MeResponse {
        authenticated: true,
        sub: payload.sub,
        email: payload.email,
        first_name: String::new(),
        last_name: String::new(),
        picture: String::new(),
        is_admin,
    })
    .into_response()
}

/// `GET|POST /auth/logout` — RP-initiated logout.
///
/// The sealed session no longer carries the id_token, so no `id_token_hint` is
/// sent; RP-initiated logout proceeds with `client_id` +
/// `post_logout_redirect_uri`. The session cookie is cleared regardless.
async fn logout(State(bff): State<Bff>, _headers: HeaderMap) -> Response {
    let cleared = [bff.cookie.clear(SESSION_COOKIE_NAME)];

    let location = match &bff.metadata.end_session_endpoint {
        Some(endpoint) => match reqwest::Url::parse(endpoint) {
            Ok(mut url) => {
                url.query_pairs_mut()
                    .append_pair("post_logout_redirect_uri", &bff.console_url)
                    .append_pair("client_id", &bff.client_id);
                url.to_string()
            }
            Err(_) => bff.console_url.clone(),
        },
        None => bff.console_url.clone(),
    };

    redirect(&location, &cleared)
}

/// Token endpoint response (only the fields the BFF needs).
#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(default)]
    id_token: Option<String>,
    /// Refresh token, sealed into the session for `/auth/refresh`.
    #[serde(default)]
    refresh_token: Option<String>,
}

/// id_token claims validated by the BFF. `aud` is kept as raw JSON because
/// providers emit it as either a string or an array.
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    iss: String,
    #[serde(default)]
    aud: serde_json::Value,
    exp: u64,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    sub: Option<String>,
}

/// Decodes the (already signature-verified) id_token payload into its claims.
fn decode_id_token_claims(token: &str) -> Result<IdTokenClaims, Error> {
    let payload = token.split('.').nth(1).ok_or(Error::MalformedIdToken)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| Error::MalformedIdToken)?;
    serde_json::from_slice(&bytes).map_err(|_| Error::MalformedIdToken)
}

/// Whether the id_token `aud` (string or array) contains `client_id`.
fn aud_contains(aud: &serde_json::Value, client_id: &str) -> bool {
    match aud {
        serde_json::Value::String(single) => single == client_id,
        serde_json::Value::Array(items) => {
            items.iter().any(|item| item.as_str() == Some(client_id))
        }
        _ => false,
    }
}

/// Reads a cookie value out of the request `Cookie` header.
fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .filter_map(|pair| pair.trim().split_once('='))
                .find(|(cookie_name, _)| *cookie_name == name)
                .map(|(_, value)| value.to_owned())
        })
        .filter(|value| !value.is_empty())
}

/// Current unix time in seconds.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Length-independent, content-constant-time string comparison, used for the
/// CSRF `state` and the `nonce`.
fn constant_time_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Builds a `HeaderValue` from an ASCII string the caller controls, falling back
/// to an empty value instead of panicking on the (unreachable) invalid case.
fn header_value(value: &str) -> HeaderValue {
    HeaderValue::from_str(value).unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// 302 redirect to `location`, attaching every `Set-Cookie` in `cookies`.
fn redirect(location: &str, cookies: &[String]) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, header_value(location));
    for cookie in cookies {
        headers.append(header::SET_COOKIE, header_value(cookie));
    }
    (StatusCode::FOUND, headers, String::new()).into_response()
}

/// Loud rejection: the given status + message, attaching any cookies to clear.
fn reject(status: StatusCode, message: &str, cookies: &[String]) -> Response {
    let mut headers = HeaderMap::new();
    for cookie in cookies {
        headers.append(header::SET_COOKIE, header_value(cookie));
    }
    (status, headers, message.to_owned()).into_response()
}
