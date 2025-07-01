use std::time::SystemTime;

tonic::include_proto!("francenuage.fr.api.controlplane.v1.infrastructure");

/// Converts a `crate::Datacenter` into a protocol compatible `v1::Datacenter`.
impl From<crate::Datacenter> for Datacenter {
    fn from(value: crate::Datacenter) -> Self {
        Datacenter {
            id: value.id.to_string(),
            name: value.name,
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}

/// Converts a `crate::ZeroTrustNetworkType` into a protocol compatible `v1::ZeroTrustNetworkType`.
impl From<crate::ZeroTrustNetworkType> for ZeroTrustNetworkType {
    fn from(value: crate::ZeroTrustNetworkType) -> Self {
        ZeroTrustNetworkType {
            id: value.id.to_string(),
            name: value.name,
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}

/// Converts a `crate::ZeroTrustNetwork` into a protocol compatible `v1::ZeroTrustNetwork`.
impl From<crate::ZeroTrustNetwork> for ZeroTrustNetwork {
    fn from(value: crate::ZeroTrustNetwork) -> Self {
        ZeroTrustNetwork {
            id: value.id.to_string(),
            organization_id: value.organization_id.to_string(),
            zero_trust_network_type_id: value.zero_trust_network_type_id.to_string(),
            name: value.name,
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}
