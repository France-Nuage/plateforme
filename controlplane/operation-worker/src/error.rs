use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialization: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("spicedb: {0}")]
    SpiceDb(#[from] spicedb::Error),
}
