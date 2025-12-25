use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),

    #[error("missing authorization header")]
    MissingAuthorizationHeader,

    #[error("malformed id {0}, expected uuid")]
    MalformedId(String),

    #[error("not found: {0}")]
    NotFound(String),
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
    fn from(_value: frn_core::Error) -> Self {
        todo!()
    }
}

impl From<Error> for tonic::Status {
    fn from(value: Error) -> Self {
        tracing::error!("oopsie from convert: {:?}", &value);
        match value {
            Error::MissingAuthorizationHeader => tonic::Status::unauthenticated(value.to_string()),
            Error::NotFound(_) => tonic::Status::not_found(value.to_string()),
            Error::MalformedId(_) => tonic::Status::invalid_argument(value.to_string()),
            _ => tonic::Status::internal(value.to_string()),
        }
    }
}
