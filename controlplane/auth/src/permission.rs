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
//! - `get` - Read a specific resource
//! - `create` - Create new resources
//! - `delete` - Delete existing resources
//!
//! ## SpiceDB Integration
//!
//! These permission strings are used directly in SpiceDB schema definitions
//! and authorization checks. The string representations must match exactly
//! with the permissions defined in the SpiceDB configuration.

use std::str::FromStr;

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
/// let permission = Permission::Get;
/// assert_eq!(permission.to_string(), "get");
/// ```
#[derive(Debug, Default, Display, EnumString)]
pub enum Permission {
    /// Permission to read a given resource.
    ///
    /// This permission allows users to read a specific resource
    /// within their authorized scope. The scope is determined by the resource
    /// relationships defined in SpiceDB.
    #[strum(serialize = "get")]
    #[default]
    Get,
}

impl From<String> for Permission {
    fn from(value: String) -> Self {
        Permission::from_str(&value)
            .unwrap_or_else(|_| panic!("invalid permission string: {}", value))
    }
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        value.to_string()
    }
}
