//! Network management module.
//!
//! Provides VPC, VNet, IPAM, and Security Group management for the France Nuage platform.
//! This module implements a comprehensive SDN (Software Defined Network) architecture
//! using Proxmox VXLAN zones for network isolation.

mod instance_interface;
mod ipam;
mod security_group;
mod vnet;
mod vpc;

pub use instance_interface::*;
pub use ipam::*;
pub use security_group::*;
pub use vnet::*;
pub use vpc::*;
