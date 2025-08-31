//! Error types for authentication operations.
//!
//! This module provides comprehensive error handling for JWT authentication and OIDC operations.
//! All errors implement the standard library's [`Error`] trait and provide detailed context for
//! debugging and monitoring.
//!
//! ## Error Categories
//!
//! The errors are organized into several categories:
//! - **Token Extraction Errors**: Issues with parsing Authorization headers
//! - **JWT Processing Errors**: Problems during token validation and decoding  
//! - **Network Errors**: Issues communicating with OIDC providers
//! - **Data Format Errors**: Problems parsing OIDC metadata or JWK sets

use thiserror::Error;
use tonic::Status;

/// Comprehensive error type for all authentication and OIDC operations.
///
/// This enum covers all possible failure modes in the authentication library,
/// from token extraction to JWT validation to OIDC provider communication.
/// Each variant provides specific context about what went wrong.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// The Authorization header contains invalid UTF-8 characters or malformed data.
    ///
    /// This occurs when the header value cannot be parsed as a valid UTF-8 string,
    /// typically due to binary data or invalid encoding in the header value.
    #[error("malformed authorization header")]
    MalformedAuthorizationHeader,

    /// No Authorization header is present in the request.
    ///
    /// This error indicates that the client did not include the required
    /// `Authorization` header in their request. The client should include
    /// a header with the format: `Authorization: Bearer <token>`
    #[error("missing authorization header")]
    MissingAuthorizationHeader,

    /// JWT header is missing the required "kid" (Key ID) claim.
    ///
    /// The JWT header must contain a "kid" field that identifies which key
    /// from the JWK Set should be used to validate the token signature.
    /// This error occurs when the JWT was signed but doesn't specify
    /// which key to use for verification.
    #[error("missing kid in jwt header")]
    MissingKid,

    /// Authorization header doesn't contain a properly formatted Bearer token.
    ///
    /// This error occurs when:
    /// - The header doesn't start with "Bearer "
    /// - The token part after "Bearer " is empty or missing
    /// - The header format is incorrect (e.g., "Basic" instead of "Bearer")
    #[error("not a bearer token")]
    MalformedBearerToken,

    /// A general error from JWT processing operations.
    ///
    /// This wraps errors from the underlying `jsonwebtoken` library,
    /// such as signature validation failures, expired tokens, or
    /// malformed JWT structure. The inner message provides specific
    /// details about what went wrong during JWT processing.
    #[error("other: {0}")]
    Other(String),

    /// Failed to parse OIDC provider metadata from the discovery endpoint.
    ///
    /// This occurs when the OIDC provider's `/.well-known/openid_configuration`
    /// endpoint returns data that cannot be parsed as valid JSON or doesn't
    /// conform to the expected OpenID Connect Discovery specification format.
    /// The URL parameter indicates which provider's metadata failed to parse.
    #[error("unparsable metadata for oidc provider {0}")]
    UnparsableOidcMetadata(String),

    /// Failed to parse the JWK Set from the provider's JWKS endpoint.
    ///
    /// This error occurs when the JWK Set retrieved from the provider's
    /// `jwks_uri` cannot be parsed as valid JSON or doesn't conform to
    /// the RFC 7517 JWK Set specification. The URL parameter indicates
    /// which JWKS endpoint failed to parse.
    #[error("unparsable jwks for url {0}")]
    UnparsableJwks(String),

    /// Cannot establish a network connection to the OIDC provider.
    ///
    /// This error occurs when HTTP requests to the OIDC provider fail
    /// due to network issues, DNS resolution problems, connection timeouts,
    /// or the provider being unavailable. The URL parameter indicates
    /// which provider endpoint was unreachable.
    #[error("unreachable oidc provider {0}")]
    UnreachableOidcProvider(String),
}

/// Converts JWT library errors into our unified error type.
///
/// This implementation allows seamless integration with the `jsonwebtoken` crate
/// by automatically converting its error types into our [`Error::Other`] variant.
/// This is particularly useful for JWT signature validation, token parsing, and
/// cryptographic operations that may fail.
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::Other(value.to_string())
    }
}

/// Converts authentication errors to tonic gRPC status responses.
///
/// This implementation provides automatic conversion from authentication errors
/// to appropriate gRPC status codes, enabling seamless integration with tonic
/// gRPC services. All authentication failures are mapped to `UNAUTHENTICATED`
/// status code as per gRPC specifications.
///
/// # gRPC Status Mapping
///
/// All `Error` variants are consistently mapped to `UNAUTHENTICATED` status:
/// - Missing tokens, invalid signatures, expired tokens, etc. all indicate
///   authentication failure and should be presented uniformly to clients
/// - Error details are preserved in the status message for debugging
impl From<Error> for tonic::Status {
    fn from(value: Error) -> Self {
        Status::unauthenticated(value.to_string())
    }
}
