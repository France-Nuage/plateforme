use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Distant instance #{0} not found.")]
    DistantInstanceNotFound(String),

    #[error("Instance {0} not found.")]
    InstanceNotFound(Uuid),

    #[error("The hypervisor {0} could not be found.")]
    HypervisorNotFound(Uuid),

    #[error("The given instance id #{0} could not be parsed into a valid uuid.")]
    MalformedInstanceId(String),

    #[error("No hypervisors are available.")]
    NoHypervisorsAvaible,

    #[error("other")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Converts a `hypervisor_connector::Problem` into a `instance::Problem`.
impl From<hypervisor_connector::Problem> for Problem {
    fn from(value: hypervisor_connector::Problem) -> Self {
        match &value {
            hypervisor_connector::Problem::DistantInstanceNotFound(id) => {
                Problem::DistantInstanceNotFound(id.to_owned())
            }
            _ => Problem::Other(Box::new(value)),
        }
    }
}

/// Converts a `sqlx::Error` into a `instance::Problem`.
impl From<sqlx::Error> for Problem {
    fn from(error: sqlx::Error) -> Self {
        Problem::Other(Box::new(error))
    }
}

/// Converts a `hypervisors::Problem` into a `instance::Problem`.
impl From<hypervisors::Problem> for Problem {
    fn from(value: hypervisors::Problem) -> Self {
        match &value {
            hypervisors::Problem::NotFound(id) => Problem::HypervisorNotFound(id.to_owned()),
            hypervisors::Problem::Other { source: _ } => Problem::Other(Box::new(value)),
        }
    }
}

/// Converts a `instance::Problem` into a `tonic::Status`.
impl From<Problem> for Status {
    fn from(value: Problem) -> Self {
        match &value {
            Problem::InstanceNotFound(_) => Status::not_found(value.to_string()),
            _ => Status::from_error(Box::new(value)),
        }
    }
}
