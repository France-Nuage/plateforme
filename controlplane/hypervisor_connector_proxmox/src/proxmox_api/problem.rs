use super::api_response::{ApiInternalErrorResponse, ApiInvalidResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Proxmox Connectivity Error: {0:?}")]
    Connectivity(#[from] reqwest::Error),

    #[error("Proxmox Internal Server Error: {}", .response.message)]
    Internal { response: ApiInternalErrorResponse },

    #[error("Proxmox Validation Error: {}", .response.message)]
    Invalid { response: ApiInvalidResponse },

    #[error("Proxmox Task #{0} has not completed")]
    TaskNotCompleted(String),

    #[error("Proxmox VM Not Found: {id}")]
    VMNotFound {
        id: String,
        response: ApiInternalErrorResponse,
    },

    #[error("Internal error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Problem> for hypervisor_connector::Problem {
    fn from(value: Problem) -> Self {
        match &value {
            Problem::VMNotFound { id, response: _ } => {
                hypervisor_connector::Problem::InstanceNotFound {
                    id: id.clone(),
                    source: Box::new(value),
                }
            }
            _ => hypervisor_connector::Problem::Other(Box::new(value)),
        }
    }
}
