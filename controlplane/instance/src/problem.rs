use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("instance {id} not found")]
    InstanceNotFound {
        id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("other")]
    Other {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Converts a `hypervisor_connector::Problem` into a `instance::Problem`.
impl From<hypervisor_connector::Problem> for Problem {
    fn from(value: hypervisor_connector::Problem) -> Self {
        match &value {
            hypervisor_connector::Problem::InstanceNotFound { id, .. } => {
                Problem::InstanceNotFound {
                    id: id.clone(),
                    source: Box::new(value),
                }
            }
            hypervisor_connector::Problem::Other(_) => Problem::Other {
                source: Box::new(value),
            },
        }
    }
}

/// Converts a `instance::Problem` into a `tonic::Status`.
impl From<Problem> for Status {
    fn from(value: Problem) -> Self {
        match &value {
            Problem::InstanceNotFound { .. } => Status::not_found(value.to_string()),
            _ => Status::from_error(Box::new(value)),
        }
    }
}
