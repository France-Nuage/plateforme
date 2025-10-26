//! Application error types
//!
//! Defines the central `Error` enum for all frn-core operations including
//! database, authorization, and configuration errors. Provides conversions from
//! SpiceDB errors and to gRPC Status codes with appropriate semantic mapping.

use thiserror::Error as ThisError;

/// Application-level errors.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Authorization server error.
    #[error("internal: {0}")]
    AuthorizationServerError(spicedb::Error),

    /// Database operation failed.
    #[error("{0}")]
    Database(#[from] sqlx::Error),

    /// Authorization check failed - access denied.
    #[error("forbidden")]
    Forbidden,

    #[error("authentication: {0}")]
    Identity(#[from] jsonwebtoken::errors::Error),

    #[error("other: {0}")]
    Other(String),

    #[error("unauthenticated")]
    Unauthenticated,

    /// Authorization builder missing principal.
    #[error("authorization check missing principal")]
    UnspecifiedPrincipal,

    /// Authorization builder missing permission.
    #[error("authorization check missing permission")]
    UnspecifiedPermission,

    /// Authorization builder missing resource.
    #[error("authorization check missing resource")]
    UnspecifiedResource,

    /// Missing required environment variable.
    #[error("missing required environment variable: {0}")]
    MissingEnvVar(String),
}

impl From<spicedb::Error> for Error {
    fn from(value: spicedb::Error) -> Self {
        match value {
            spicedb::Error::Forbidden => Error::Forbidden,
            error => Error::AuthorizationServerError(error),
        }
    }
}

impl From<Error> for tonic::Status {
    fn from(value: Error) -> tonic::Status {
        match value {
            Error::Unauthenticated => tonic::Status::unauthenticated(value.to_string()),
            Error::Forbidden => tonic::Status::permission_denied(value.to_string()),
            err => {
                tracing::error!("internal error: {}", err);
                tonic::Status::internal("internal error")
            }
        }
    }
}
