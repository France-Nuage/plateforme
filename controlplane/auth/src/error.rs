//! Error types for authentication and authorization operations.
//!
//! This module provides comprehensive error handling for JWT authentication, OIDC operations,
//! and SpiceDB authorization checks. All errors implement the standard library's [`Error`] trait
//! and provide detailed context for debugging and monitoring.
//!
//! ## Error Categories
//!
//! The errors are organized into several categories:
//! - **Token Extraction Errors**: Issues with parsing Authorization headers
//! - **JWT Processing Errors**: Problems during token validation and decoding
//! - **Network Errors**: Issues communicating with OIDC providers or SpiceDB
//! - **Data Format Errors**: Problems parsing OIDC metadata or JWK sets
//! - **Authorization Errors**: SpiceDB permission check failures and configuration errors
//! - **Database Errors**: Temporary category for user authorization lookup failures (will be removed with SpiceDB)

use http::uri::InvalidUri;
use thiserror::Error;
use tonic::Status;

/// Comprehensive error type for all authentication and OIDC operations.
///
/// This enum covers all possible failure modes in the authentication library,
/// from token extraction to JWT validation to OIDC provider communication.
/// Each variant provides specific context about what went wrong.
#[derive(Debug, Error)]
pub enum Error {
    /// Error communicating with the SpiceDB authorization server.
    ///
    /// This error occurs when gRPC requests to the SpiceDB server fail due to
    /// network issues, server unavailability, or protocol errors. The contained
    /// message provides details from the underlying gRPC error.
    ///
    /// Common causes include:
    /// - SpiceDB server is not running or unreachable
    /// - Network connectivity issues
    /// - gRPC protocol errors or version mismatches
    /// - Server-side processing failures
    #[error("authorization server error: {0}")]
    AuthorizationServerError(String),

    /// Database query error during user authorization lookup.
    ///
    /// This error occurs when database operations fail during the user authorization
    /// process, such as connection failures, query execution errors, or constraint
    /// violations. This is a temporary error type that will be removed once SpiceDB
    /// integration replaces database-backed authorization with stateless lookups.
    ///
    /// **Note**: This error variant is temporary and will be removed when migrating
    /// from database-backed user authorization to SpiceDB.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("forbidden")]
    Forbidden,

    #[error("internal: {0}")]
    Internal(String),

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

    /// JWT token is missing the required "email" claim for user authorization.
    ///
    /// This error occurs when a JWT token is successfully validated but lacks
    /// the "email" claim needed to identify the user in the authorization system.
    /// The email claim is used to look up user records in the database to determine
    /// organizational membership and access permissions.
    ///
    /// **Note**: This error is specific to the current database-backed authorization
    /// model and will be replaced by SpiceDB subject-based authorization.
    #[error("missing email claim")]
    MissingEmailClaim,

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

    /// The provided SpiceDB server URL cannot be parsed as a valid URI.
    ///
    /// This error occurs during client initialization when the SpiceDB server
    /// URL string cannot be parsed into a valid URI format. This typically
    /// indicates malformed URLs missing schemes, invalid characters, or
    /// incorrect formatting.
    ///
    /// # Examples of Invalid URLs
    /// - Missing scheme: `spicedb:50051` (should be `http://spicedb:50051`)
    /// - Invalid characters: `http://spice db:50051` (spaces not allowed)
    /// - Malformed port: `http://spicedb:abc` (port must be numeric)
    #[error("unparsable authz server url: {0}")]
    UnparsableAuthzServerUrl(#[from] InvalidUri),

    #[error("the authz token could not be parsed to a grpc metadata")]
    UnparsableAuthzToken,

    /// Cannot establish connection to the SpiceDB authorization server.
    ///
    /// This error occurs when attempting to connect to the SpiceDB server
    /// fails due to network issues, server unavailability, or connection
    /// timeouts. The contained URL shows which server was unreachable.
    ///
    /// Common causes include:
    /// - SpiceDB server is not running
    /// - Network connectivity issues
    /// - Incorrect server URL or port
    /// - Firewall or security group blocking connections
    #[error("unreachable authz server ({0})")]
    UnreachableAuthzServer(String),

    /// Authorization check attempted without specifying a permission.
    #[error("no permission was specified for this authorization request")]
    UnspecifiedPermission,

    #[error("no relation was specified for this authorization request")]
    UnspecifedRelation,

    /// Authorization check attempted without specifying a target resource.
    #[error("no resource was specified for this authorization request")]
    UnspecifiedResource,

    /// Authorization check attempted without specifying a subject (user).
    #[error("no subject was specified for this authorization request")]
    UnspecifiedSubject,

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

    /// Authenticated user is not registered in the authorization system.
    ///
    /// This error occurs when a JWT token is successfully validated and contains
    /// a valid email claim, but no corresponding user record exists in the database.
    /// This typically indicates that the user exists in the identity provider (GitLab)
    /// but hasn't been provisioned in the controlplane authorization system.
    ///
    /// The contained string value is the email address from the JWT token that
    /// couldn't be found in the user registry.
    ///
    /// **Note**: This error is specific to the current database-backed authorization
    /// and will be replaced by SpiceDB relationship-based access control.
    #[error("user {0} is not registered")]
    UserNotRegistered(String),
}

/// Converts JWT library errors into our unified error type.
///
/// This implementation allows seamless integration with the `jsonwebtoken` crate
/// by automatically converting its error types into our [`Error::Other`] variant.
/// This is particularly useful for JWT signature validation, token parsing, and
/// cryptographic operations that may fail.
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        Error::MalformedBearerToken
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
