//! Error types for the Pangolin API.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Pangolin API connectivity error: {0}")]
    Connectivity(#[from] reqwest::Error),

    #[error("Pangolin API unauthorized - check PANGOLIN_API_KEY")]
    Unauthorized,

    #[error("Pangolin API resource not found: {0}")]
    NotFound(String),

    #[error("Pangolin API bad request: {0}")]
    BadRequest(String),

    #[error("Pangolin API conflict: {0}")]
    Conflict(String),

    #[error("Pangolin API internal error: {0}")]
    Internal(String),

    #[error("Pangolin API unexpected response: {0}")]
    UnexpectedResponse(String),
}
