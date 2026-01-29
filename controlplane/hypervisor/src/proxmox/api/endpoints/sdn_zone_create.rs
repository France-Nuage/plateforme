use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Creates a new SDN zone in the Proxmox cluster.
///
/// A zone defines the isolation technology (VXLAN) and contains the VNI
/// (unique tunnel identifier) along with the list of peers (hypervisor IPs).
pub async fn sdn_zone_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    config: &SdnZoneConfig,
) -> Result<ApiResponse<()>, Error> {
    client
        .post(format!("{}/api2/json/cluster/sdn/zones", api_url))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(config)
        .send()
        .await
        .to_api_response()
        .await
}

/// Configuration for creating an SDN zone.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdnZoneConfig {
    /// Unique zone identifier.
    pub zone: String,

    /// Zone type. Use "vxlan" for VXLAN overlay networks.
    #[serde(rename = "type")]
    pub zone_type: SdnZoneType,

    /// Comma-separated list of peer IP addresses (hypervisors in the cluster).
    pub peers: Option<String>,

    /// MTU for the zone. Defaults to 1450 for VXLAN.
    pub mtu: Option<u32>,

    /// VXLAN Network Identifier (VNI). Must be unique across zones.
    /// Range: 1-16777215.
    #[serde(rename = "vxlan-port")]
    pub vxlan_port: Option<u16>,
}

/// SDN zone types supported by Proxmox.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SdnZoneType {
    /// Simple zone without isolation.
    Simple,
    /// VLAN-based isolation.
    Vlan,
    /// VXLAN overlay network for cross-node isolation.
    Vxlan,
    /// QinQ double-tagging.
    Qinq,
    /// EVPN for advanced routing.
    Evpn,
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnZoneCreateMock {
        fn with_sdn_zone_create(self) -> Self;
    }

    impl WithSdnZoneCreateMock for MockServer {
        fn with_sdn_zone_create(mut self) -> Self {
            let mock = self
                .server
                .mock("POST", "/api2/json/cluster/sdn/zones")
                .with_body(r#"{"data":null}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithSdnZoneCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_zone_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_zone_create();
        let config = SdnZoneConfig {
            zone: "testzone".to_string(),
            zone_type: SdnZoneType::Vxlan,
            peers: Some("10.0.0.1,10.0.0.2".to_string()),
            mtu: Some(1450),
            vxlan_port: None,
        };
        let result = sdn_zone_create(&server.url(), &client, "", &config).await;
        assert!(result.is_ok());
    }
}
