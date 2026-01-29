//! SDN Subnet creation for Proxmox.
//!
//! Subnets define IP ranges within VNets and optionally enable DHCP for automatic IP assignment.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Creates a new SDN subnet in the Proxmox cluster.
///
/// A subnet defines the IP range (CIDR) and gateway associated with a VNet.
/// This is used for IPAM (IP Address Management).
pub async fn sdn_subnet_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    vnet_id: &str,
    config: &SdnSubnetConfig,
) -> Result<ApiResponse<()>, Error> {
    client
        .post(format!(
            "{}/api2/json/cluster/sdn/vnets/{}/subnets",
            api_url, vnet_id
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdnSubnetConfig {
    /// Subnet identifier in CIDR format (e.g., "10.0.0.0-24").
    /// Note: Proxmox uses hyphen instead of slash in the subnet ID.
    pub subnet: String,

    /// Gateway IP address for the subnet.
    pub gateway: Option<String>,

    /// Enable SNAT (Source NAT) for this subnet.
    pub snat: Option<bool>,

    /// DHCP range start address.
    #[serde(rename = "dhcp-range")]
    pub dhcp_range: Option<String>,

    /// DNS zone name for this subnet.
    #[serde(rename = "dnszoneprefix")]
    pub dns_zone_prefix: Option<String>,
}

impl SdnSubnetConfig {
    /// Creates a subnet config from a CIDR string.
    /// Converts "10.0.0.0/24" to the Proxmox format "10.0.0.0-24".
    pub fn from_cidr(cidr: &str, gateway: Option<String>) -> Self {
        let subnet = cidr.replace('/', "-");
        Self {
            subnet,
            gateway,
            snat: None,
            dhcp_range: None,
            dns_zone_prefix: None,
        }
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnSubnetCreateMock {
        fn with_sdn_subnet_create(self) -> Self;
    }

    impl WithSdnSubnetCreateMock for MockServer {
        fn with_sdn_subnet_create(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(
                        r"^/api2/json/cluster/sdn/vnets/.*/subnets$".to_string(),
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
    use super::mock::WithSdnSubnetCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_subnet_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_subnet_create();
        let config = SdnSubnetConfig::from_cidr("10.0.0.0/24", Some("10.0.0.1".to_string()));
        let result = sdn_subnet_create(&server.url(), &client, "", "testvnet", &config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_cidr_conversion() {
        let config = SdnSubnetConfig::from_cidr("192.168.1.0/24", Some("192.168.1.1".to_string()));
        assert_eq!(config.subnet, "192.168.1.0-24");
        assert_eq!(config.gateway, Some("192.168.1.1".to_string()));
    }
}
