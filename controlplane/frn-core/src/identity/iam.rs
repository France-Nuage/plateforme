//! Identity and access management
//!
//! Provides the `IAM` service for resolving user identity from access tokens.
//! Currently returns a default user; will be extended to validate OIDC tokens.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    Error,
    identity::{Principal, ServiceAccount, SessionKey, User},
};
use auth::OpenID;
use fabrique::Query;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct IAM {
    pub db: Pool<Postgres>,
    pub identity: OpenID,
    /// Key that opens BFF session cookies. `None` when the confidential-client
    /// BFF is not configured (no `AUTH_COOKIE_KEY`) — in that case the cookie
    /// credential path is unavailable and requests must carry a bearer token.
    pub session_key: Option<SessionKey>,
}

impl IAM {
    pub fn new(db: Pool<Postgres>, identity: OpenID, session_key: Option<SessionKey>) -> Self {
        Self {
            db,
            identity,
            session_key,
        }
    }

    /// Resolves the [`Principal`] behind a gRPC request.
    ///
    /// Two credential paths, tried in order (bearer wins when both are present):
    ///
    /// 1. **`Authorization: Bearer <token>`** — a service-account key (matched by
    ///    KEY first, so SAs are never affected by the cookie path), otherwise a
    ///    user OIDC access token validated by [`OpenID::validate_token`]
    ///    (signature **and** expiry).
    /// 2. **`frn_session` cookie** — our own sealed [`SessionPayload`], **not** a
    ///    JWT. It is opened with the server key, its short inner `exp` is
    ///    enforced, and the user is resolved by email. A decrypt failure
    ///    (tampered, wrong key, garbage) or an elapsed `exp` fails closed as
    ///    [`Error::Unauthenticated`] — never a 500/panic.
    pub async fn principal<T>(&self, request: &tonic::Request<T>) -> Result<Principal, Error> {
        if let Some(bearer) = bearer_token(request) {
            if let Some(service_account) = ServiceAccount::query()
                .select()
                .r#where(ServiceAccount::KEY, "=", bearer.clone())
                .first(&self.db)
                .await?
            {
                return Ok(Principal::ServiceAccount(service_account));
            }

            return self.user(bearer).await.map(Principal::User);
        }

        if let Some(cookie) = session_token_from_cookie(request) {
            return self.principal_from_session(&cookie).await.map(Principal::User);
        }

        Err(Error::Unauthenticated)
    }

    /// Opens the sealed session cookie and resolves its user, failing closed on
    /// any decrypt/parse error or an elapsed inner `exp`.
    async fn principal_from_session(&self, cookie: &str) -> Result<User, Error> {
        let session_key = self.session_key.as_ref().ok_or(Error::Unauthenticated)?;
        let payload = session_key.open(cookie).map_err(|_| Error::Unauthenticated)?;

        if payload.exp <= unix_now() {
            return Err(Error::Unauthenticated);
        }

        User::find_or_create_one_by_email(&self.db, &payload.email)
            .await
            .map_err(Into::into)
    }

    #[cfg(test)]
    /// Test-only accessor exercising the cookie credential extraction in
    /// isolation (the browser → gRPC-web → tonic path is covered end-to-end by
    /// the server crate's black-box tests).
    pub fn session_token_from_cookie_for_test<T>(request: &tonic::Request<T>) -> Option<String> {
        session_token_from_cookie(request)
    }

    async fn user(&self, access_token: String) -> Result<User, Error> {
        let email = self
            .identity
            .validate_token(&access_token)
            .await?
            .claims
            .email
            .ok_or(auth::Error::MissingEmailClaim)?;

        User::find_or_create_one_by_email(&self.db, &email)
            .await
            .map_err(Into::into)
    }
}

/// Extracts a `Bearer` token from the request's `Authorization` metadata.
fn bearer_token<T>(request: &tonic::Request<T>) -> Option<String> {
    request
        .metadata()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|value| value.to_owned())
        .filter(|token| !token.is_empty())
}

/// Extracts the sealed BFF session cookie value from the request's `Cookie`
/// header.
///
/// Returns the value of the [`crate::identity::SESSION_COOKIE_NAME`] cookie when
/// present. The value is our own encrypted [`crate::identity::SessionPayload`]
/// (opened by [`SessionKey`]), not a JWT: no trust is granted by mere presence.
fn session_token_from_cookie<T>(request: &tonic::Request<T>) -> Option<String> {
    request
        .metadata()
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .filter_map(|pair| pair.trim().split_once('='))
                .find(|(name, _)| *name == crate::identity::SESSION_COOKIE_NAME)
                .map(|(_, value)| value.to_owned())
        })
        .filter(|token| !token.is_empty())
}

/// Current unix time in seconds.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_cookie(cookie: &str) -> tonic::Request<()> {
        let mut request = tonic::Request::new(());
        request.metadata_mut().insert(
            "cookie",
            cookie.parse().expect("cookie header must be valid ascii"),
        );
        request
    }

    #[test]
    fn extracts_the_session_token_from_a_lone_cookie() {
        let request = request_with_cookie("frn_session=header.payload.signature");
        assert_eq!(
            IAM::session_token_from_cookie_for_test(&request),
            Some("header.payload.signature".to_owned())
        );
    }

    #[test]
    fn extracts_the_session_token_among_several_cookies() {
        let request = request_with_cookie("theme=dark; frn_session=abc.def.ghi; locale=fr");
        assert_eq!(
            IAM::session_token_from_cookie_for_test(&request),
            Some("abc.def.ghi".to_owned())
        );
    }

    #[test]
    fn ignores_an_empty_session_cookie() {
        let request = request_with_cookie("frn_session=");
        assert_eq!(IAM::session_token_from_cookie_for_test(&request), None);
    }

    #[test]
    fn returns_none_without_a_session_cookie() {
        let request = request_with_cookie("theme=dark; locale=fr");
        assert_eq!(IAM::session_token_from_cookie_for_test(&request), None);
    }
}
