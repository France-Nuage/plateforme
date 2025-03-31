use thiserror::Error;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Instance Not Found: {id}")]
    InstanceNotFound {
        id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
