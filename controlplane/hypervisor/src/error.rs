use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Distant instance #{0} not found.")]
    DistantInstanceNotFound(String),

    #[error("Distant instance #{0} not running.")]
    InstanceNotRunning(String),

    #[error("The value {0} could not be parsed to a valid vm id.")]
    MalformedVmId(String),

    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
