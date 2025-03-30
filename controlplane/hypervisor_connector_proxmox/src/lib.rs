mod instance_service;
pub mod proxmox_api;

#[cfg(feature = "mock")]
pub mod mock;

pub use instance_service::ProxmoxInstanceService;
