//! Permission definitions for fine-grained access control.
//!
//! This module defines the permission types used throughout the authorization
//! system. Each permission corresponds to a specific operation that can be
//! performed on resources within the platform.
//!
//! ## Design Principles
//!
//! Permissions follow a hierarchical naming convention inspired by Google Cloud IAM:
//! - **Domain**: The service or resource family (e.g., `compute`)
//! - **Resource**: The specific resource type (e.g., `instances`)
//! - **Action**: The operation being performed (e.g., `list`, `create`, `delete`)
//!
//! ## Format
//!
//! Permissions use dot notation: `domain.resource.action`
//!
//! Examples:
//! - `compute.instances.list` - List compute instances
//! - `compute.instances.create` - Create new compute instances
//! - `storage.buckets.read` - Read from storage buckets
//!
//! ## SpiceDB Integration
//!
//! These permission strings are used directly in SpiceDB schema definitions
//! and authorization checks. The string representations must match exactly
//! with the permissions defined in the SpiceDB configuration.

use strum_macros::{Display, EnumString};

/// Enumeration of all permissions available in the platform.
///
/// Each variant represents a specific permission that can be granted to users
/// or roles within the authorization system. The string representation of each
/// permission (via the `Display` trait) is used in SpiceDB authorization checks.
///
/// ## Examples
///
/// ```
/// use auth::Permission;
///
/// let permission = Permission::ListInstances;
/// assert_eq!(permission.to_string(), "compute.instances.list");
/// ```
#[derive(Display, EnumString)]
pub enum Permission {
    /// Permission to list compute instances.
    ///
    /// This permission allows users to retrieve a list of compute instances
    /// within their authorized scope. The scope is determined by the resource
    /// relationships defined in SpiceDB.
    #[strum(serialize = "compute.instances.list")]
    ListInstances,
}
