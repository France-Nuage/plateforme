use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("instance {id} not found")]
    InstanceNotFound {
        id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error(
        "Invalid distant id: the instance {instance_id} distant id {distant_id} could not be parsed into a proxmox id."
    )]
    InvalidDistantId {
        distant_id: String,
        instance_id: Uuid,
    },

    #[error(
        "The hypervisor {hypervisor_id} attached to the instance {instance_id} could not be found."
    )]
    HypervisorNotFound {
        hypervisor_id: Uuid,
        instance_id: Uuid,
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

/// Converts a `sqlx::Error` into a `instance::Problem`.
impl From<sqlx::Error> for Problem {
    fn from(error: sqlx::Error) -> Self {
        match &error {
            sqlx::Error::RowNotFound => Problem::InstanceNotFound {
                id: String::from(""),
                source: Box::new(error),
            },
            _ => Problem::Other {
                source: Box::new(error),
            },
        }
    }
}

/// Converts a `hypervisors::Problem` into a `instance::Problem`.
impl From<hypervisors::Problem> for Problem {
    fn from(value: hypervisors::Problem) -> Self {
        match &value {
            hypervisors::Problem::NotFound => Problem::HypervisorNotFound {
                hypervisor_id: Uuid::default(),
                instance_id: Uuid::default(),
            },
            hypervisors::Problem::Other { source: _ } => Problem::Other {
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
