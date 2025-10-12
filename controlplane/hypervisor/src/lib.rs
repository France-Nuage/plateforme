mod error;
pub mod instance;
pub mod proxmox;
mod resolver;

#[cfg(feature = "mock")]
pub mod mock;

pub use error::*;
pub use resolver::resolve;
