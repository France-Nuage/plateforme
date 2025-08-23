//! # Auth - JWT Authentication and OIDC Discovery Library
//!
//! A comprehensive authentication library providing JWT token validation and OpenID Connect (OIDC)
//! discovery capabilities for Rust applications. This crate is designed for server applications that
//! need to validate JWT tokens from OIDC providers.
//!
//! ## Overview
//!
//! This library provides:
//! - **JWT Token Extraction**: Extract Bearer tokens from tonic gRPC requests
//! - **OIDC Discovery**: Automatically discover OpenID Connect provider configuration
//! - **JWT Validation**: Validate JWT tokens using JWK (JSON Web Key) sets with caching
//! - **Standards Compliance**: Full compliance with RFC 7517 (JWK), RFC 7519 (JWT), and OIDC specifications
//!
//! ## Design Principles
//!
//! - **Security First**: Implements secure token validation with proper key management
//! - **Performance**: Uses intelligent caching for JWK keys to minimize network requests
//! - **Standards Compliant**: Adheres to IETF and OpenID Foundation specifications
//! - **Async Ready**: Built on tokio for high-performance async operations
//! - **Error Transparency**: Comprehensive error types for debugging and monitoring
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use auth::{JwkValidator, extract_authorization_token};
//! use tonic::Request;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize validator with OIDC discovery
//! let validator = JwkValidator::from_oidc_discovery(
//!     "https://accounts.google.com/.well-known/openid_configuration"
//! ).await?;
//!
//! // Extract token from tonic request (in your gRPC service)
//! # let request: Request<()> = Request::new(());
//! let token = extract_authorization_token(request)?;
//!
//! // Validate the token
//! let claims = validator.validate_token(&token).await?;
//! println!("Token valid for subject: {:?}", claims.claims.sub);
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! This crate consists of several internal modules:
//! - **discovery** - OIDC provider metadata and discovery functionality
//! - **error** - Comprehensive error types for all authentication operations  
//! - **rfc7517** - JWT claims structures following RFC 7517 specification
//! - **validator** - JWT validation with JWK key management and caching
//!
//! ## Features
//!
//! ### Token Extraction
//! Extract JWT tokens from HTTP Authorization headers in tonic gRPC requests:
//! ```rust,no_run
//! use auth::extract_authorization_token;
//! use tonic::Request;
//!
//! # fn example(request: Request<()>) -> Result<(), auth::Error> {
//! let token = extract_authorization_token(request)?;
//! // Use token for validation...
//! # Ok(())
//! # }
//! ```
//!
//! ### OIDC Discovery
//! Automatically discover provider configuration from well-known endpoints:
//! ```rust,no_run
//! # async fn example() -> Result<(), auth::Error> {
//! let validator = auth::JwkValidator::from_oidc_discovery(
//!     "https://login.microsoftonline.com/common/.well-known/openid_configuration"
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### JWT Validation
//! Validate tokens with automatic key fetching and caching:
//! ```rust,no_run
//! # async fn example(validator: auth::JwkValidator, token: String) -> Result<(), auth::Error> {
//! let token_data = validator.validate_token(&token).await?;
//!
//! // Access standard claims
//! if let Some(expiry) = token_data.claims.exp {
//!     println!("Token expires at: {}", expiry);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Standards Compliance
//!
//! This library implements the following specifications:
//! - [RFC 7517](https://tools.ietf.org/html/rfc7517) - JSON Web Key (JWK)
//! - [RFC 7519](https://tools.ietf.org/html/rfc7519) - JSON Web Token (JWT)  
//! - [OpenID Connect Discovery 1.0](https://openid.net/specs/openid-connect-discovery-1_0.html)
//!
//! ## Caching Strategy
//!
//! The library uses intelligent caching for JWK keys:
//! - **Capacity**: Up to 200 keys cached simultaneously
//! - **TTL**: 1-hour time-to-live for each key
//! - **Concurrent Fetching**: Efficient parallel key fetching with backpressure control

pub use error::Error;
use tonic::Request;
pub use validator::JwkValidator;

mod discovery;
mod error;
mod rfc7517;
mod validator;

/// Extracts JWT token from a tonic gRPC request's Authorization header.
///
/// This function searches for the `authorization` metadata in a tonic [`Request`] and parses
/// it according to the Bearer token authentication scheme defined in [RFC 6750].
///
/// ## Format Expected
///
/// The Authorization header must follow the format: `Bearer <token>`
/// where `<token>` is a valid JWT token string.
///
/// ## Arguments
///
/// * `req` - A tonic [`Request`] containing the gRPC request metadata
///
/// ## Returns
///
/// * `Ok(String)` - The extracted JWT token string if successful
/// * `Err(Error)` - An error if the header is missing, malformed, or doesn't contain a valid Bearer token
///
/// ## Errors
///
/// This function will return an error in the following cases:
///
/// * [`Error::MissingAuthorizationHeader`] - No `authorization` header is present
/// * [`Error::MalformedAuthorizationHeader`] - The header contains invalid UTF-8 characters
/// * [`Error::MalformedBearerToken`] - The header doesn't start with "Bearer " or the token is empty
///
/// ## Examples
///
/// ```rust
/// use tonic::{Request, metadata::MetadataValue};
/// use auth::extract_authorization_token;
///
/// # fn example() -> Result<(), auth::Error> {
/// // Create a request with a valid Bearer token
/// let mut request = Request::new(());
/// request.metadata_mut().insert(
///     "authorization",
///     "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".parse().unwrap()
/// );
///
/// let token = extract_authorization_token(request)?;
/// println!("Extracted token: {}", token);
/// # Ok(())
/// # }
/// ```
///
/// ## Security Considerations
///
/// This function only extracts the token - it does not validate its signature or claims.
/// Always validate extracted tokens using [`JwkValidator::validate_token`] before trusting
/// their contents.
///
/// [RFC 6750]: https://tools.ietf.org/html/rfc6750
/// [`JwkValidator::validate_token`]: crate::JwkValidator::validate_token
pub fn extract_authorization_token<T>(req: Request<T>) -> Result<String, Error> {
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
    //! Unit tests for JWT token extraction functionality.
    //!
    //! These tests verify the correct behavior of the `extract_authorization_token`
    //! function across various scenarios including valid tokens, missing headers,
    //! and malformed data.

    use tonic::metadata::MetadataValue;

    use super::*;

    /// Sample JWT token for testing purposes.
    ///
    /// This is a valid JWT structure but uses a test signature that should not be
    /// used for validation in production environments. The token contains standard
    /// claims for testing token extraction logic.
    const TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30";

    /// Tests successful extraction of a Bearer token from a valid Authorization header.
    ///
    /// This test verifies that when a properly formatted "Bearer <token>" header
    /// is present in a tonic request, the token portion is correctly extracted
    /// without the "Bearer " prefix.
    #[test]
    fn test_the_authorization_token_extraction() {
        // Arrange a tonic request with a valid authorization header
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", TOKEN).parse().unwrap(),
        );

        // Act the call to the extraction function
        let result = extract_authorization_token(request);

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TOKEN);
    }

    /// Tests that missing Authorization header returns the appropriate error.
    ///
    /// This test ensures that when no Authorization header is present in the
    /// request metadata, the function returns `Error::MissingAuthorizationHeader`
    /// rather than panicking or returning an incorrect error type.
    #[test]
    fn test_a_missing_authorization_header_produces_an_error() {
        // Arrange a tonic request with no authorization header
        let request = Request::new(());

        // Act the call to the extraction function
        let result = extract_authorization_token(request);

        // Assert the result
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingAuthorizationHeader);
    }

    /// Tests that malformed Authorization header with invalid UTF-8 returns appropriate error.
    ///
    /// This test verifies error handling when the Authorization header contains
    /// invalid UTF-8 characters that cannot be converted to a string. This scenario
    /// can occur in practice due to encoding issues or malicious requests.
    #[test]
    fn test_a_malformed_authorization_header_produces_an_error() {
        // Arrange a tonic request with a malformed authorization header
        let mut request = Request::new(());
        let malformed_header = unsafe { format!("Bearer {}token", char::from_u32_unchecked(128)) };
        let value = MetadataValue::try_from(malformed_header).unwrap();
        request.metadata_mut().insert("authorization", value);

        // Act the call t the extraction function
        let result = extract_authorization_token(request);
        println!("result: {:?}", &result);

        // Assert the result
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MalformedAuthorizationHeader);
    }
}
