//! Target backend for operations
//!
//! Defines the external systems that operations can target for synchronization.
//! Each backend corresponds to an external service that the control plane
//! needs to keep in sync with its internal state.

use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};

/// External systems that operations can target.
///
/// Each backend represents an external service requiring synchronization:
/// - **SpiceDb**: Authorization service (relationships and permissions)
/// - **Pangolin**: Zero Trust VPN for user access management
/// - **Hoop**: SSH bastion for VM access tunnels
/// - **Kubernetes**: Kubernetes namespace access for multi-tenant clusters
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetBackend {
    /// SpiceDB authorization service (replaces authz-worker).
    #[default]
    SpiceDb,
    /// Pangolin Zero Trust VPN for user access.
    Pangolin,
    /// Hoop SSH bastion for VM access.
    Hoop,
    /// Kubernetes namespace access management.
    Kubernetes,
}

impl From<String> for TargetBackend {
    fn from(value: String) -> Self {
        TargetBackend::from_str(&value).expect("could not parse target backend")
    }
}

impl From<TargetBackend> for String {
    fn from(value: TargetBackend) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_display() {
        assert_eq!(TargetBackend::SpiceDb.to_string(), "SPICE_DB");
        assert_eq!(TargetBackend::Pangolin.to_string(), "PANGOLIN");
        assert_eq!(TargetBackend::Hoop.to_string(), "HOOP");
        assert_eq!(TargetBackend::Kubernetes.to_string(), "KUBERNETES");
    }

    #[test]
    fn test_backend_from_string() {
        assert_eq!(
            TargetBackend::from_str("SPICE_DB").unwrap(),
            TargetBackend::SpiceDb
        );
        assert_eq!(
            TargetBackend::from_str("PANGOLIN").unwrap(),
            TargetBackend::Pangolin
        );
    }
}
