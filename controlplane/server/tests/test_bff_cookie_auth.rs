//! Black-box tests for the BFF → gRPC bridge (the cookie credential path).
//!
//! Under the confidential-client flow the browser holds no bearer token: its
//! gRPC-web calls carry the httpOnly `frn_session` cookie, which is our own
//! **encrypted, self-contained** session (not a JWT). These tests exercise the
//! control plane opening that sealed cookie in `IAM::principal`, enforcing its
//! short inner `exp`, and failing closed on tampering — while the bearer path
//! (service accounts, user id_tokens) keeps working with expiry now enforced.

mod common;

use std::time::{SystemTime, UNIX_EPOCH};

use common::{Api, OnBehalfOf, WithUser};
use frn_core::identity::{SessionKey, SessionPayload, TEST_SESSION_KEY};
use frn_rpc::v1::iam::GetCurrentUserRequest;
use frn_rpc::v1::resourcemanager::CreateOrganizationRequest;
use tonic::metadata::MetadataValue;
use tonic::{Code, Request};

const EMAIL: &str = "cookie-user@francenuage.fr";

/// Current unix time in seconds.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

/// Seals a session cookie with the same deterministic key the test server's IAM
/// opens with (`App::test` → `TEST_SESSION_KEY`).
fn seal(email: &str, exp: u64) -> String {
    SessionKey::from_bytes(TEST_SESSION_KEY)
        .seal(&SessionPayload {
            refresh_token: "rt".to_owned(),
            sub: "subject-1".to_owned(),
            email: email.to_owned(),
            exp,
        })
        .expect("could not seal session cookie")
}

/// Attaches a raw BFF session cookie value (and nothing else) to a gRPC request.
trait WithSessionCookie {
    fn with_session_cookie(self, cookie: &str) -> Self;
}

impl<T> WithSessionCookie for Request<T> {
    fn with_session_cookie(mut self, cookie: &str) -> Self {
        let value = MetadataValue::try_from(format!("frn_session={cookie}"))
            .expect("could not build cookie metadata value");
        self.metadata_mut().insert("cookie", value);
        self
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn a_sealed_session_cookie_authenticates_a_grpc_call(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    // The cookie is our encrypted session blob, opened server-side by IAM.
    let cookie = seal(EMAIL, now() + 3600);

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_session_cookie(&cookie))
        .await
        .expect("session cookie should authenticate")
        .into_inner();

    assert_eq!(response.email, EMAIL);
    assert!(!response.is_admin);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn an_empty_session_cookie_is_rejected(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_session_cookie(""))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn an_expired_sealed_session_is_rejected(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    // A well-formed, correctly-sealed cookie whose inner access exp has elapsed.
    let cookie = seal(EMAIL, now() - 3600);

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_session_cookie(&cookie))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn a_tampered_session_cookie_is_rejected(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // Garbage that cannot be decrypted must fail closed as unauthenticated, never
    // a 500/panic.
    let response = api
        .profile
        .get_current_user(
            Request::new(GetCurrentUserRequest {}).with_session_cookie("not-a-sealed-token"),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn an_expired_bearer_id_token_is_rejected(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // A validly-signed but expired user id_token presented as a bearer token must
    // be rejected now that `validate_exp` is on.
    let expired = auth::OpenID::sign_claims(&serde_json::json!({
        "email": EMAIL,
        "exp": now() - 3600,
        "iat": now() - 7200,
        "nbf": now() - 7200,
    }));

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_user(&expired))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn a_bearer_id_token_with_a_verified_email_authenticates(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // A validly-signed, non-expired user token with a verified email resolves the
    // user by email — the positive counterpart to the rejection below.
    let token = auth::OpenID::sign_claims(&serde_json::json!({
        "email": EMAIL,
        "email_verified": true,
        "exp": now() + 3600,
        "iat": now(),
        "nbf": now(),
    }));

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_user(&token))
        .await;

    assert!(response.is_ok(), "a verified-email bearer must authenticate");
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn a_bearer_id_token_with_an_unverified_email_is_rejected(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // Validly-signed and non-expired, but `email_verified` is false: since users
    // are resolved by email, an unverified address must NOT authenticate (it could
    // belong to someone else — an admin — the attacker never proved control of).
    let token = auth::OpenID::sign_claims(&serde_json::json!({
        "email": EMAIL,
        "email_verified": false,
        "exp": now() + 3600,
        "iat": now(),
        "nbf": now(),
    }));

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_user(&token))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn a_service_account_bearer_still_authenticates(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // Service accounts are matched by KEY before the user path, so the sealed
    // cookie rework leaves them unaffected.
    let request = Request::new(CreateOrganizationRequest {
        name: String::from("ACME"),
        parent_slug: None,
    })
    .on_behalf_of(&api.service_account);

    let response = api.resourcemanager.organizations.create(request).await;

    assert!(response.is_ok(), "service-account bearer must authenticate");
    Ok(())
}
