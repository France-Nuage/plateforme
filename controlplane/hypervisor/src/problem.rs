use thiserror::Error;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("instance {vm_id} not found")]
    InstanceNotFound {
        vm_id: String,
        source: Box<dyn std::error::Error>,
    },

    #[error("other")]
    Other { source: Box<dyn std::error::Error> },
}
