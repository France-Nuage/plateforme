//! SDN VNet creation endpoint for Proxmox VE.
//!
//! Creates a VNet (virtual network) within an SDN zone.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Creates a new SDN VNet in Proxmox.
///
/// API: POST /cluster/sdn/vnets
pub async fn sdn_vnet_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    config: &SDNVNetConfig,
) -> Result<ApiResponse<Option<String>>, Error> {
    client
        .post(format!("{}/api2/json/cluster/sdn/vnets", api_url))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(config)
        .send()
        .await
        .to_api_response()
        .await
}

/// Configuration for creating an SDN VNet.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct SDNVNetConfig {
    /// VNet identifier (becomes the bridge name, e.g., "vnet-myapp")
    pub vnet: String,

    /// Zone this VNet belongs to
    pub zone: String,

    /// Alias/description for the VNet
    pub alias: Option<String>,

    /// VLAN tag (optional, for VLAN-based zones)
    pub tag: Option<u32>,
}

impl SDNVNetConfig {
    /// Creates a new VNet configuration.
    pub fn new(vnet: String, zone: String) -> Self {
        Self {
            vnet,
            zone,
            alias: None,
            tag: None,
        }
    }

    /// Sets an alias/description.
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias = Some(alias);
        self
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSDNVNetCreateMock {
        fn with_sdn_vnet_create(self) -> Self;
    }

    impl WithSDNVNetCreateMock for MockServer {
        fn with_sdn_vnet_create(mut self) -> Self {
            let mock = self
                .server
                .mock("POST", "/api2/json/cluster/sdn/vnets")
                .with_body(r#"{"data":null}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithSDNVNetCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_vnet_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_vnet_create();
        let config = SDNVNetConfig::new("vnet-test".to_string(), "zone-test".to_string());
        let result = sdn_vnet_create(&server.url(), &client, "", &config).await;

        assert!(result.is_ok());
    }
}
