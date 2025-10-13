use thiserror::Error as ThisError;
use tonic::Status;
use uuid::Uuid;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Distant instance #{0} not found.")]
    DistantInstanceNotFound(String),

    #[error("Instance {0} not found.")]
    InstanceNotFound(Uuid),

    #[error("Instance {0} not started.")]
    InstanceNotStarted(Uuid),

    #[error("The hypervisor {0} could not be found.")]
    HypervisorNotFound(Uuid),

    #[error("The given instance id #{0} could not be parsed into a valid uuid.")]
    MalformedInstanceId(String),

    #[error("No hypervisors are available.")]
    NoHypervisorsAvaible,

    #[error("Other")]
    Other(Box<dyn std::error::Error + Send + Sync>),

    #[error("Unexpected instance status, got '{0}'")]
    UnexpectedInstanceStatus(String),
}

/// Converts a `hypervisor_connector::Problem` into a `instance::Problem`.
impl From<hypervisor::Error> for Error {
    fn from(value: hypervisor::Error) -> Self {
        match &value {
            hypervisor::Error::DistantInstanceNotFound(id) => {
                Error::DistantInstanceNotFound(id.to_owned())
            }
            _ => Error::Other(Box::new(value)),
        }
    }
}

/// Converts a `sqlx::Error` into a `instance::Problem`.
impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::Other(Box::new(error))
    }
}

/// Converts a `hypervisors::Problem` into a `instance::Problem`.
impl From<frn_core::Error> for Error {
    fn from(value: frn_core::Error) -> Self {
        Error::Other(Box::new(value))
    }
}

/// Converts a `instance::Problem` into a `tonic::Status`.
impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match &value {
            Error::InstanceNotFound(_) => Status::not_found(value.to_string()),
            _ => Status::from_error(Box::new(value)),
        }
    }
}
