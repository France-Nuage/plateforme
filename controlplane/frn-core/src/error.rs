use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),

    #[error("other")]
    Other,
}
