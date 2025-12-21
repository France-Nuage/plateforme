//! Error types for the Hoop.dev API.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Hoop API connectivity error: {0}")]
    Connectivity(#[from] reqwest::Error),

    #[error("Hoop API unauthorized - check HOOP_API_KEY")]
    Unauthorized,

    #[error("Hoop API resource not found: {0}")]
    NotFound(String),

    #[error("Hoop API bad request: {0}")]
    BadRequest(String),

    #[error("Hoop API internal error: {0}")]
    Internal(String),

    #[error("Hoop API unexpected response: {0}")]
    UnexpectedResponse(String),
}
