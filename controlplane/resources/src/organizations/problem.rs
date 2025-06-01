use thiserror::Error;
use tonic::{Code, Status};

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Organization not found: {0}")]
    OrganizationNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid organization data: {0}")]
    InvalidOrganizationData(String),

    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Problem> for Status {
    fn from(problem: Problem) -> Self {
        match problem {
            Problem::OrganizationNotFound(id) => {
                Status::new(Code::NotFound, format!("Organization not found: {}", id))
            }
            Problem::DatabaseError(msg) => {
                Status::new(Code::Internal, format!("Database error: {}", msg))
            }
            Problem::InvalidOrganizationData(msg) => Status::new(
                Code::InvalidArgument,
                format!("Invalid organization data: {}", msg),
            ),
            Problem::Other(err) => Status::new(Code::Unknown, format!("Unknown error: {}", err)),
        }
    }
}
