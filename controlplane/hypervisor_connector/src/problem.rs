use thiserror::Error;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Distant instance #{0} not found.")]
    DistantInstanceNotFound(String),
    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
