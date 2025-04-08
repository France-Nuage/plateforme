//! Error types for the IAM crate.

use tonic::Status;

/// Errors that can occur during authentication and authorization.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The token is invalid or has expired.
    #[error("invalid token: {0}")]
    InvalidToken(String),

    /// The token has expired.
    #[error("token has expired")]
    TokenExpired,

    /// Access was denied due to insufficient permissions.
    #[error("access denied: {0}")]
    AccessDenied(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Error during HTTP request.
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Error during JWT processing.
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Provider-specific error.
    #[error("provider error: {0}")]
    ProviderError(String),

    /// Internal server error.
    #[error("internal server error: {0}")]
    InternalError(String),
}

impl From<AuthError> for Status {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                Status::unauthenticated(error.to_string())
            }
            AuthError::AccessDenied(_) => Status::permission_denied(error.to_string()),
            AuthError::ConfigError(_) => Status::failed_precondition(error.to_string()),
            _ => Status::internal(error.to_string()),
        }
    }
}

/// Result type for authentication operations.
pub type AuthResult<T> = Result<T, AuthError>;