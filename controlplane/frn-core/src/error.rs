use thiserror::Error as ThisError;

/// Application-level errors.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Database operation failed.
    #[error("{0}")]
    Database(#[from] sqlx::Error),

    /// Authorization check failed - access denied.
    #[error("forbidden")]
    Forbidden,

    /// Authorization server error.
    #[error("internal: {0}")]
    AuthorizationServerError(spicedb::Error),
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
        tonic::Status::internal("oopsie")
    }
}
