use thiserror::Error;

use crate::api_response::{ApiInternalErrorResponse, ApiInvalidResponse};

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Proxmox Connectivity Error: {0:?}")]
    Connectivity(#[from] reqwest::Error),

    #[error("Proxmox Internal Server Error: {}", .response.message)]
    Internal { response: ApiInternalErrorResponse },

    #[error("Proxmox Validation Error: {}", .response.message)]
    Invalid { response: ApiInvalidResponse },

    #[error("Proxmox VM Not Found: {id}")]
    VMNotFound {
        id: String,
        response: ApiInternalErrorResponse,
    },

    #[error("Internal error: {0}")]
    Other(Box<dyn std::error::Error>),
}

impl From<Problem> for hypervisor::problem::Problem {
    fn from(value: Problem) -> Self {
        match &value {
            Problem::VMNotFound { id, response: _ } => {
                hypervisor::problem::Problem::InstanceNotFound {
                    vm_id: id.clone(),
                    source: Box::new(value),
                }
            }
            _ => hypervisor::problem::Problem::Other {
                source: Box::new(value),
            },
        }
    }
}
