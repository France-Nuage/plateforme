use super::{
    api_response::{ApiInternalErrorResponse, ApiInvalidResponse},
    cluster_resources_list::ResourceType,
};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Proxmox Connectivity Error: {0:?}")]
    Connectivity(#[from] reqwest::Error),

    #[error("The resource is guarded by Cloudflare")]
    GuardedByCloudflare,

    #[error("Proxmox Internal Server Error: {}", .response.message)]
    Internal { response: ApiInternalErrorResponse },

    #[error("Proxmox Validation Error: {}", .response.message)]
    Invalid { response: ApiInvalidResponse },

    #[error("Proxmox Agent missing or not configured")]
    MissingAgent,

    #[error("No nodes are available on the cluster.")]
    NoNodesAvailable,

    #[error("Proxmox resource #{id} of type {resource_type} is missing field {field}")]
    ResourceMissingField {
        id: String,
        resource_type: ResourceType,
        field: String,
    },

    #[error("Proxmox Task #{0} has not completed")]
    TaskNotCompleted(String),

    #[error("Proxmox Unauthorized Error")]
    Unauthorized,

    #[error("Unexpected redirect: #{0}")]
    UnexpectedRedirect(Url),

    #[error("Proxmox VM Not Found: {0}")]
    VMNotFound(u32),

    #[error("Proxmox VM Not Running: {0}")]
    VMNotRunning(u32),

    #[error("Internal error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        match &value {
            Error::VMNotFound(id) => crate::Error::DistantInstanceNotFound(id.to_string()),
            Error::VMNotRunning(id) => crate::Error::InstanceNotRunning(id.to_string()),
            _ => crate::Error::Other(Box::new(value)),
        }
    }
}
