//! # Hypervisor
//!
//! This crate provides a gRPC service for managing hypervisors in the France Nuage platform.
//! It allows listing registered hypervisors and registering new hypervisors.
//!
//! The primary components are:
//! - A model for hypervisor data persistence
//! - A gRPC service implementation that handles hypervisor operations
//! - Type conversions between model and API types
//! - Error handling for hypervisor operations

pub mod model;
pub mod problem;
pub mod rpc;
mod service;
pub mod v1;

pub use model::Model;
pub use service::HypervisorsService;
