use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("forbidden")]
    Forbidden,

    #[error("internal: {0}")]
    Internal(String),

    #[error("spicedb error: {0}")]
    ServerError(#[from] tonic::Status),

    #[error("unparsable spicedb token")]
    UnparsableToken,

    #[error("unparsable spicedb url")]
    UnparsableUrl,

    #[error("unreachable spicedb server")]
    UnreachableServer,
}
