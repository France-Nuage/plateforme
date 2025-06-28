use thiserror::Error;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Other")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Converts a `sqlx::Error` into a `infrastructure::Problem`.
impl From<sqlx::Error> for Problem {
    fn from(value: sqlx::Error) -> Self {
        Problem::Other(Box::new(value))
    }
}

/// Converts a `infrastructure::Problem` into a `tonic::Status`.
impl From<Problem> for tonic::Status {
    fn from(value: Problem) -> Self {
        tonic::Status::from_error(Box::new(value))
    }
}
