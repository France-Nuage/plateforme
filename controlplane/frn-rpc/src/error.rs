use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),

    #[error("forbidden")]
    Forbidden,

    #[error("internal: {0}")]
    Internal(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("missing authorization header")]
    MissingAuthorizationHeader,

    #[error("malformed id {0}, expected uuid")]
    MalformedId(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("unauthenticated")]
    Unauthenticated,
}

impl Error {
    pub fn convert(error: frn_core::Error) -> tonic::Status {
        Error::from(error).into()
    }
    pub fn sqlx_to_status(error: sqlx::Error) -> tonic::Status {
        Error::from(error).into()
    }
}

impl From<frn_core::Error> for Error {
    fn from(value: frn_core::Error) -> Self {
        match value {
            frn_core::Error::Database(e) => Error::Database(e),
            frn_core::Error::Forbidden => Error::Forbidden,
            frn_core::Error::InvalidArgument(msg) => Error::InvalidArgument(msg),
            frn_core::Error::PermissionDenied(msg) => Error::PermissionDenied(msg),
            frn_core::Error::SlugAlreadyExists(slug) => Error::AlreadyExists(slug),
            frn_core::Error::Unauthenticated => Error::Unauthenticated,
            frn_core::Error::AuthenticationServerError(_) => Error::Unauthenticated,
            other => Error::Internal(other.to_string()),
        }
    }
}

impl From<Error> for tonic::Status {
    fn from(value: Error) -> Self {
        match value {
            Error::MissingAuthorizationHeader | Error::Unauthenticated => {
                tonic::Status::unauthenticated(value.to_string())
            }
            Error::Forbidden => tonic::Status::permission_denied(value.to_string()),
            Error::PermissionDenied(_) => tonic::Status::permission_denied(value.to_string()),
            Error::NotFound(_) => tonic::Status::not_found(value.to_string()),
            Error::MalformedId(_) | Error::InvalidArgument(_) => {
                tonic::Status::invalid_argument(value.to_string())
            }
            Error::AlreadyExists(_) => tonic::Status::already_exists(value.to_string()),
            Error::Database(_) | Error::Internal(_) => {
                tracing::error!("internal error during gRPC conversion: {:?}", &value);
                tonic::Status::internal("internal error")
            }
        }
    }
}
