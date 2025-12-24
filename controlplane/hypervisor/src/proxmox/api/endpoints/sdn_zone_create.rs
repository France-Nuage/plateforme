//! SDN Zone creation endpoint for Proxmox VE.
//!
//! Creates a VXLAN zone in the Proxmox SDN configuration.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Creates a new SDN zone in Proxmox.
///
/// API: POST /cluster/sdn/zones
pub async fn sdn_zone_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    config: &SDNZoneConfig,
) -> Result<ApiResponse<Option<String>>, Error> {
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
#[derive(Debug, Serialize)]
pub struct SDNZoneConfig {
    /// Zone identifier (e.g., "zone-myapp")
    pub zone: String,

    /// Zone type - "vxlan" for VXLAN zones
    #[serde(rename = "type")]
    pub zone_type: String,

    /// Comma-separated list of peer IP addresses for VXLAN
    pub peers: Option<String>,

    /// Bridge device (e.g., "vmbr0")
    pub bridge: Option<String>,

    /// Maximum Transmission Unit (1280-1500)
    pub mtu: Option<u32>,

    /// VXLAN tag (VNI) for network isolation
    #[serde(rename = "vxlan-tag")]
    pub vxlan_tag: Option<u32>,

    /// IPAM (IP Address Management) - optional
    pub ipam: Option<String>,

    /// DNS server for the zone
    pub dns: Option<String>,

    /// Reverse DNS zone
    pub reversedns: Option<String>,

    /// DHCP setting
    pub dhcp: Option<String>,
}

impl SDNZoneConfig {
    /// Creates a new VXLAN zone configuration.
    pub fn new_vxlan(zone: String, vxlan_tag: u32, mtu: u32) -> Self {
        Self {
            zone,
            zone_type: "vxlan".to_string(),
            peers: None,
            bridge: Some("vmbr0".to_string()),
            mtu: Some(mtu),
            vxlan_tag: Some(vxlan_tag),
            ipam: None,
            dns: None,
            reversedns: None,
            dhcp: None,
        }
    }

    /// Sets the peer addresses for VXLAN zone.
    pub fn with_peers(mut self, peers: Vec<String>) -> Self {
        self.peers = Some(peers.join(","));
        self
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSDNZoneCreateMock {
        fn with_sdn_zone_create(self) -> Self;
    }

    impl WithSDNZoneCreateMock for MockServer {
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
    use super::mock::WithSDNZoneCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_zone_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_zone_create();
        let config = SDNZoneConfig::new_vxlan("zone-test".to_string(), 100, 1450);
        let result = sdn_zone_create(&server.url(), &client, "", &config).await;

        assert!(result.is_ok());
    }
}
