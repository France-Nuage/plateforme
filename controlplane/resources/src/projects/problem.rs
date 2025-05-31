use thiserror::Error;
use tonic::{Code, Status};

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Organization not found: {0}")]
    OrganizationNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid project data: {0}")]
    InvalidProjectData(String),

    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Converts a `sqlx::Error` into a `resources::projects::Problem`.
impl From<sqlx::Error> for Problem {
    fn from(error: sqlx::Error) -> Self {
        Problem::Other(Box::new(error))
    }
}

/// Converts a `resources::projects::Problem` into a `tonic::Status`.
impl From<Problem> for Status {
    fn from(problem: Problem) -> Self {
        match problem {
            Problem::ProjectNotFound(id) => {
                Status::new(Code::NotFound, format!("Project not found: {}", id))
            }
            Problem::OrganizationNotFound(id) => {
                Status::new(Code::NotFound, format!("Organization not found: {}", id))
            }
            Problem::DatabaseError(msg) => {
                Status::new(Code::Internal, format!("Database error: {}", msg))
            }
            Problem::InvalidProjectData(msg) => Status::new(
                Code::InvalidArgument,
                format!("Invalid project data: {}", msg),
            ),
            Problem::Other(err) => Status::new(Code::Unknown, format!("Unknown error: {}", err)),
        }
    }
}
