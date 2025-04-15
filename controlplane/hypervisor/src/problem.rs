use sea_orm::DbErr;
use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("hypervisor not found")]
    NotFound,
    #[error("other")]
    Other {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<DbErr> for Problem {
    fn from(value: DbErr) -> Self {
        match value {
            DbErr::RecordNotFound(_) => Problem::NotFound,
            _ => Problem::Other {
                source: Box::new(value),
            },
        }
    }
}

impl From<Problem> for Status {
    fn from(value: Problem) -> Self {
        match value {
            Problem::NotFound => Status::not_found(value.to_string()),
            _ => Status::from_error(Box::new(value)),
        }
    }
}
