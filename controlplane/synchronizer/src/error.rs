use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("core error: {0}")]
    Core(#[from] frn_core::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("hypervisor error: {0}")]
    Hypervisor(#[from] hypervisor::Error),
}
