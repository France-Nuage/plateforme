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
//! - **Authentication Middleware**: Tower middleware layer for automatic request authentication
//! - **IAM Context**: Per-request identity and access management context
//! - **SpiceDB Authorization**: Fine-grained access control with Google Zanzibar-style permissions
//! - **Permission System**: Type-safe permission definitions with fluent authorization API
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
//! ### Manual Token Validation
//!
//! ```
//! use auth::{OpenID, extract_authorization_token};
//! use tonic::Request;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize OpenID with OIDC discovery
//! let openid = OpenID::discover(
//!     reqwest::Client::new(),
//!     "https://accounts.google.com/.well-known/openid_configuration"
//! ).await?;
//!
//! // Extract token from tonic request (in your gRPC service)
//! # let request: Request<()> = Request::new(());
//! let token = extract_authorization_token(&request)?;
//!
//! // Validate the token
//! let claims = openid.validate_token(&token).await?;
//! println!("Token valid for subject: {:?}", claims.claims.sub);
//! # Ok(())
//! # }
//! ```
//!
//! ### Middleware Integration
//!
//! ```rust
//! use auth::{AuthenticationLayer, Authz, OpenID};
//! use tower::ServiceBuilder;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let openid = OpenID::discover(
//!     reqwest::Client::new(),
//!     "https://provider.com/.well-known/openid_configuration"
//! ).await?;
//! let authz = Authz::mock().await;
//!
//! let service = ServiceBuilder::new().layer(AuthenticationLayer::new(authz, openid));
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! This crate consists of several modules:
//! - **authentication_layer** - Tower middleware for HTTP request authentication
//! - **iam** - Identity and Access Management context for authenticated requests
//! - **authz** - SpiceDB authorization client with fluent API for permission checking
//! - **permission** - Type-safe permission definitions for fine-grained access control
//! - **openid** - JWT token validation with JWK key management and caching
//! - **error** - Comprehensive error types for all authentication and authorization operations
//! - **rfc7517** - JWT claims structures following RFC 7517 specification
//!
//! ## Features
//!
//! ### Token Extraction
//! Extract JWT tokens from HTTP Authorization headers in tonic gRPC requests:
//! ```
//! use auth::extract_authorization_token;
//! use tonic::Request;
//!
//! # fn example(request: Request<()>) -> Result<(), auth::Error> {
//! let token = extract_authorization_token(&request)?;
//! // Use token for validation...
//! # Ok(())
//! # }
//! ```
//!
//! ### OIDC Discovery
//! Automatically discover provider configuration from well-known endpoints:
//! ```rust,no_run
//! # async fn example() -> Result<(), auth::Error> {
//! let openid = auth::OpenID::discover(
//!     reqwest::Client::new(),
//!     "https://login.microsoftonline.com/common/.well-known/openid_configuration"
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### JWT Validation
//! Validate tokens with automatic key fetching and caching:
//! ```rust,no_run
//! # async fn example(openid: auth::OpenID, token: String) -> Result<(), auth::Error> {
//! let token_data = openid.validate_token(&token).await?;
//!
//! // Access standard claims
//! if let Some(expiry) = token_data.claims.exp {
//!     println!("Token expires at: {}", expiry);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Authorization with SpiceDB
//! Perform fine-grained access control checks using SpiceDB:
//! ```
//! # use auth::{Authz, Permission};
//! # use frn_core::identity::User;
//! # use uuid::Uuid;
//! # async fn example() -> Result<(), auth::Error> {
//! let authz = Authz::connect("http://spicedb:50051".to_owned(), "Bearer f00ba3".to_owned()).await?;
//! let user = User::default();
//! let instance_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
//!
//! // Check if user can read an instance
//! authz
//!     .can(&user)
//!     .perform(Permission::Get)
//!     .on(("instance", &instance_id))
//!     .check()
//!     .await?;
//!
//! println!("User authorized to read instance");
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

pub use authentication_layer::AuthenticationLayer;
pub use authorize::Authorize;
pub use authz::Authz;
pub use derive_auth::Authorize;
pub use error::Error;
pub use iam::IAM;
pub use openid::OpenID;
pub use permission::Permission;
pub use relationship_queue::RELATIONSHIP_QUEUE_NAME;
pub use relationship_queue::Relationship;
use tonic::Request;

mod authentication_layer;
mod authorize;
mod authz;
mod error;
pub mod iam;
mod openid;
mod permission;
mod relationship_queue;
mod rfc7519;

#[cfg(feature = "mock")]
pub mod mock;

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
/// let token = extract_authorization_token(&request)?;
/// println!("Extracted token: {}", token);
/// # Ok(())
/// # }
/// ```
///
/// ## Security Considerations
///
/// This function only extracts the token - it does not validate its signature or claims.
/// Always validate extracted tokens using [`OpenID::validate_token`] before trusting
/// their contents.
///
/// [RFC 6750]: https://tools.ietf.org/html/rfc6750
/// [`OpenID::validate_token`]: crate::OpenID::validate_token
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
        let result = extract_authorization_token(&request);

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
        let result = extract_authorization_token(&request);

        // Assert the result
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::MissingAuthorizationHeader
        ));
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
        let result = extract_authorization_token(&request);

        // Assert the result
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::MalformedAuthorizationHeader
        ));
    }
}
