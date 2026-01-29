use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Creates a new SDN VNet in the Proxmox cluster.
///
/// A VNet is a virtual bridge attached to a zone. It's the name used when
/// attaching VMs (e.g., `net0: bridge=<vnet>`). VNets appear on each node
/// after applying the SDN configuration.
pub async fn sdn_vnet_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    config: &SdnVnetConfig,
) -> Result<ApiResponse<()>, Error> {
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdnVnetConfig {
    /// Unique VNet identifier. This becomes the bridge name for VM attachment.
    pub vnet: String,

    /// Zone this VNet belongs to.
    pub zone: String,

    /// Optional alias/description for the VNet.
    pub alias: Option<String>,

    /// VXLAN Network Identifier (tag). Must be unique within the zone.
    /// Range: 1-16777215.
    pub tag: Option<u32>,

    /// Enable VLAN awareness on the bridge.
    pub vlanaware: Option<bool>,
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnVnetCreateMock {
        fn with_sdn_vnet_create(self) -> Self;
    }

    impl WithSdnVnetCreateMock for MockServer {
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
    use super::mock::WithSdnVnetCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_vnet_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_vnet_create();
        let config = SdnVnetConfig {
            vnet: "testvnet".to_string(),
            zone: "testzone".to_string(),
            alias: Some("Test VNet".to_string()),
            tag: Some(100),
            vlanaware: None,
        };
        let result = sdn_vnet_create(&server.url(), &client, "", &config).await;
        assert!(result.is_ok());
    }
}
