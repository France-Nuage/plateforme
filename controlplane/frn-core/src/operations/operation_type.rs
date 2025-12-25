//! Operation type definitions
//!
//! Defines all operation types supported by the operations system.
//! Each type corresponds to a specific action on a target backend.

use super::TargetBackend;
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};

/// Types of operations that can be executed against external systems.
///
/// Operations are grouped by target backend:
///
/// ## SpiceDB Operations
/// - `SpiceDbWriteRelationship`: Create a relationship tuple
/// - `SpiceDbDeleteRelationship`: Remove a relationship tuple
///
/// ## Pangolin Operations
/// - `PangolinInviteUser`: Invite a user to an organization
/// - `PangolinRemoveUser`: Remove a user from an organization
/// - `PangolinUpdateUser`: Update user information in an organization
///
/// ## Hoop Operations
/// - `HoopCreateAgent`: Create an SSH agent for a VM
/// - `HoopDeleteAgent`: Delete an SSH agent
/// - `HoopCreateConnection`: Create an SSH connection/tunnel
/// - `HoopDeleteConnection`: Delete an SSH connection
///
/// ## Kubernetes Operations
/// - `K8sCreateNamespaceAccess`: Grant user access to a namespace
/// - `K8sDeleteNamespaceAccess`: Revoke user access from a namespace
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationType {
    // SpiceDB operations (replaces authz-worker)
    /// Write a relationship tuple to SpiceDB.
    #[default]
    SpiceDbWriteRelationship,
    /// Delete a relationship tuple from SpiceDB.
    SpiceDbDeleteRelationship,

    // Pangolin operations (user lifecycle)
    /// Invite a user to a Pangolin organization.
    PangolinInviteUser,
    /// Remove a user from a Pangolin organization.
    PangolinRemoveUser,
    /// Update user information in Pangolin.
    PangolinUpdateUser,

    // Hoop operations (SSH access)
    /// Create an SSH agent in Hoop.
    HoopCreateAgent,
    /// Delete an SSH agent from Hoop.
    HoopDeleteAgent,
    /// Create an SSH connection in Hoop.
    HoopCreateConnection,
    /// Delete an SSH connection from Hoop.
    HoopDeleteConnection,

    // Kubernetes operations (namespace access)
    /// Create namespace access for a user.
    K8sCreateNamespaceAccess,
    /// Delete namespace access for a user.
    K8sDeleteNamespaceAccess,
}

impl OperationType {
    /// Returns the target backend for this operation type.
    pub fn target_backend(&self) -> TargetBackend {
        match self {
            OperationType::SpiceDbWriteRelationship | OperationType::SpiceDbDeleteRelationship => {
                TargetBackend::SpiceDb
            }
            OperationType::PangolinInviteUser
            | OperationType::PangolinRemoveUser
            | OperationType::PangolinUpdateUser => TargetBackend::Pangolin,
            OperationType::HoopCreateAgent
            | OperationType::HoopDeleteAgent
            | OperationType::HoopCreateConnection
            | OperationType::HoopDeleteConnection => TargetBackend::Hoop,
            OperationType::K8sCreateNamespaceAccess | OperationType::K8sDeleteNamespaceAccess => {
                TargetBackend::Kubernetes
            }
        }
    }
}

impl From<String> for OperationType {
    fn from(value: String) -> Self {
        OperationType::from_str(&value).expect("could not parse operation type")
    }
}

impl From<OperationType> for String {
    fn from(value: OperationType) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type_display() {
        assert_eq!(
            OperationType::SpiceDbWriteRelationship.to_string(),
            "SPICE_DB_WRITE_RELATIONSHIP"
        );
        assert_eq!(
            OperationType::PangolinInviteUser.to_string(),
            "PANGOLIN_INVITE_USER"
        );
        assert_eq!(
            OperationType::HoopCreateAgent.to_string(),
            "HOOP_CREATE_AGENT"
        );
    }

    #[test]
    fn test_target_backend() {
        assert_eq!(
            OperationType::SpiceDbWriteRelationship.target_backend(),
            TargetBackend::SpiceDb
        );
        assert_eq!(
            OperationType::PangolinInviteUser.target_backend(),
            TargetBackend::Pangolin
        );
        assert_eq!(
            OperationType::HoopCreateAgent.target_backend(),
            TargetBackend::Hoop
        );
        assert_eq!(
            OperationType::K8sCreateNamespaceAccess.target_backend(),
            TargetBackend::Kubernetes
        );
    }
}
