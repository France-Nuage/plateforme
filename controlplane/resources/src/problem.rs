use thiserror::Error;
use tonic::{Code, Status};

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Converts a `sqlx::Error` into a `resources::Problem`.
impl From<sqlx::Error> for Problem {
    fn from(error: sqlx::Error) -> Self {
        Problem::Other(Box::new(error))
    }
}

/// Converts a `resources::Problem` into a `tonic::Status`.
impl From<Problem> for Status {
    fn from(problem: Problem) -> Self {
        match problem {
            Problem::Other(err) => Status::new(Code::Unknown, format!("Unknown error: {}", err)),
        }
    }
}
