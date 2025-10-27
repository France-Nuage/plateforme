//! JWT authentication and authorization for gRPC services.
//!
//! Provides JWT token validation with OpenID Connect discovery and permission checking via SpiceDB.
//!
//! ## Quick Start
//!
//!
//! ### OIDC Discovery
//!
//! Automatically discover provider configuration from well-known endpoints:
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), auth::Error> {
//! let openid = auth::OpenID::discover(
//!     reqwest::Client::new(),
//!     "https://accounts.google.com/.well-known/openid-configuration"
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### JWT Validation
//!
//! Validate tokens with automatic key fetching and caching:
//!
//! ```rust,no_run
//! # async fn example(openid: auth::OpenID, token: String) -> Result<(), auth::Error> {
//! let token_data = openid.validate_token(&token).await?;
//!
//! // Access standard claims
//! if let Some(email) = token_data.claims.email {
//!     println!("Token for user: {}", email);
//! }
//! # Ok(())
//! # }
//! ```

pub use authorize::Authorize;
pub use derive_auth::Authorize;
pub use error::Error;
pub use openid::OpenID;
pub use permission::Permission;
use tonic::Request;

mod authorize;
mod error;
mod openid;
mod permission;
mod rfc7519;

#[cfg(feature = "mock")]
pub mod mock;

/// Extracts JWT token from a tonic gRPC request's Authorization header.
///
/// Parses the `authorization` metadata according to the Bearer token authentication
/// scheme defined in RFC 6750.
///
/// ## Format Expected
///
/// The Authorization header must follow the format: `Bearer <token>`
/// where `<token>` is a valid JWT token string.
///
/// ## Returns
///
/// * `Ok(String)` - The extracted JWT token string
/// * `Err(Error)` - If the header is missing, malformed, or doesn't contain a valid Bearer token
///
/// ## Errors
///
/// * [`Error::MissingAuthorizationHeader`] - No `authorization` header is present
/// * [`Error::MalformedAuthorizationHeader`] - The header contains invalid UTF-8 characters
/// * [`Error::MalformedBearerToken`] - The header doesn't start with "Bearer " or the token is empty
///
/// ## Security
///
/// This function only extracts the token - it does not validate its signature or claims.
/// Always validate extracted tokens using [`OpenID::validate_token`] before trusting their contents.
pub fn extract_authorization_token<T>(req: &Request<T>) -> Result<String, Error> {
    req.metadata()
        .get("authorization")
        .ok_or(Error::MissingAuthorizationHeader)?
        .to_str()
        .map_err(|_| Error::MalformedAuthorizationHeader)?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .ok_or(Error::MalformedBearerToken)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    const TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30";

    #[test]
    fn test_the_authorization_token_extraction() {
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", TOKEN).parse().unwrap(),
        );

        let result = extract_authorization_token(&request);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TOKEN);
    }

    #[test]
    fn test_a_missing_authorization_header_produces_an_error() {
        let request = Request::new(());
        let result = extract_authorization_token(&request);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::MissingAuthorizationHeader
        ));
    }

    #[test]
    fn test_a_malformed_authorization_header_produces_an_error() {
        let mut request = Request::new(());
        let malformed_header = unsafe { format!("Bearer {}token", char::from_u32_unchecked(128)) };
        let value = MetadataValue::try_from(malformed_header).unwrap();
        request.metadata_mut().insert("authorization", value);

        let result = extract_authorization_token(&request);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::MalformedAuthorizationHeader
        ));
    }
}
