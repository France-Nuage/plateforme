//! Operations infrastructure for async external system synchronization
//!
//! This module provides the foundation for reliable asynchronous operations
//! that synchronize the Control Plane state with external systems. It implements
//! a GCP-style Long Running Operations pattern with:
//!
//! - **State machine**: PENDING → RUNNING → SUCCEEDED/FAILED/CANCELLED
//! - **Retry with exponential backoff**: Automatic retries with configurable limits
//! - **Concurrent processing**: `FOR UPDATE SKIP LOCKED` for safe worker concurrency
//! - **Pluggable executors**: Support for multiple backends via `OperationExecutor` trait
//!
//! ## Supported Backends
//!
//! | Backend | Purpose | Operations |
//! |---------|---------|------------|
//! | SpiceDB | Authorization | WriteRelationship, DeleteRelationship |
//! | Pangolin | Zero Trust VPN | InviteUser, RemoveUser, UpdateUser |
//! | Hoop | SSH Bastion | CreateAgent, DeleteAgent, CreateConnection, DeleteConnection |
//! | Kubernetes | Namespace Access | CreateNamespaceAccess, DeleteNamespaceAccess |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use frn_core::operations::{Operation, OperationType};
//!
//! // Create an operation
//! let operation = Operation::new(
//!     OperationType::PangolinInviteUser,
//!     "Invitation",
//!     invitation.id,
//!     serde_json::json!({
//!         "org_slug": organization.slug,
//!         "email": user.email,
//!     }),
//! )
//! .create(&pool)
//! .await?;
//! ```

mod executor;
mod operation;
mod operation_type;
mod retry;
mod status;
mod target_backend;

pub use executor::*;
pub use operation::*;
pub use operation_type::*;
pub use retry::*;
pub use status::*;
pub use target_backend::*;
