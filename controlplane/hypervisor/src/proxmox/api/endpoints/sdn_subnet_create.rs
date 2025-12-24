//! SDN Subnet creation endpoint for Proxmox VE.
//!
//! Creates a subnet within an SDN VNet.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Creates a new SDN subnet in Proxmox.
///
/// API: POST /cluster/sdn/vnets/{vnet}/subnets
pub async fn sdn_subnet_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    vnet: &str,
    config: &SDNSubnetConfig,
) -> Result<ApiResponse<Option<String>>, Error> {
    client
        .post(format!(
            "{}/api2/json/cluster/sdn/vnets/{}/subnets",
            api_url, vnet
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(config)
        .send()
        .await
        .to_api_response()
        .await
}

/// Configuration for creating an SDN subnet.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct SDNSubnetConfig {
    /// Subnet identifier in CIDR format (e.g., "10.0.1.0-24" or "10.0.1.0/24")
    /// Note: Proxmox uses "-" instead of "/" in the subnet identifier
    pub subnet: String,

    /// Gateway IP address for this subnet
    pub gateway: Option<String>,

    /// Enable SNAT (Source NAT) for outbound traffic
    pub snat: Option<bool>,

    /// DHCP range start IP
    #[serde(rename = "dhcp-range")]
    pub dhcp_range: Option<String>,

    /// DNS zone file for this subnet
    #[serde(rename = "dnszoneprefix")]
    pub dns_zone_prefix: Option<String>,
}

impl SDNSubnetConfig {
    /// Creates a new subnet configuration.
    ///
    /// The subnet should be in CIDR notation (e.g., "10.0.1.0/24").
    /// It will be converted to Proxmox format internally.
    pub fn new(subnet: String, gateway: String) -> Self {
        // Convert CIDR format to Proxmox format (replace / with -)
        let proxmox_subnet = subnet.replace('/', "-");

        Self {
            subnet: proxmox_subnet,
            gateway: Some(gateway),
            snat: Some(true),
            dhcp_range: None,
            dns_zone_prefix: None,
        }
    }

    /// Disables SNAT.
    pub fn without_snat(mut self) -> Self {
        self.snat = Some(false);
        self
    }

    /// Sets DHCP range.
    pub fn with_dhcp_range(mut self, start: &str, end: &str) -> Self {
        self.dhcp_range = Some(format!("{}-{}", start, end));
        self
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSDNSubnetCreateMock {
        fn with_sdn_subnet_create(self) -> Self;
    }

    impl WithSDNSubnetCreateMock for MockServer {
        fn with_sdn_subnet_create(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(
                        r"^/api2/json/cluster/sdn/vnets/.+/subnets$".to_string(),
                    ),
                )
                .with_body(r#"{"data":null}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithSDNSubnetCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_subnet_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_subnet_create();
        let config = SDNSubnetConfig::new("10.0.1.0/24".to_string(), "10.0.1.1".to_string());
        let result = sdn_subnet_create(&server.url(), &client, "", "vnet-test", &config).await;

        assert!(result.is_ok());
    }
}
