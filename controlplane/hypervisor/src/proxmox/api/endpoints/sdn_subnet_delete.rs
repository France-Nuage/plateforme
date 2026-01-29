//! SDN Subnet deletion for Proxmox.
//!
//! Removes a subnet from a VNet. The subnet must not have active DHCP leases.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Deletes an SDN subnet from the Proxmox cluster.
pub async fn sdn_subnet_delete(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    vnet_id: &str,
    subnet_id: &str,
) -> Result<ApiResponse<()>, Error> {
    client
        .delete(format!(
            "{}/api2/json/cluster/sdn/vnets/{}/subnets/{}",
            api_url, vnet_id, subnet_id
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

/// Converts a CIDR to Proxmox subnet ID format.
/// "10.0.0.0/24" becomes "10.0.0.0-24".
pub fn cidr_to_subnet_id(cidr: &str) -> String {
    cidr.replace('/', "-")
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnSubnetDeleteMock {
        fn with_sdn_subnet_delete(self) -> Self;
    }

    impl WithSdnSubnetDeleteMock for MockServer {
        fn with_sdn_subnet_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(
                        r"^/api2/json/cluster/sdn/vnets/.*/subnets/.*$".to_string(),
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
    use super::mock::WithSdnSubnetDeleteMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_subnet_delete() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_subnet_delete();
        let result =
            sdn_subnet_delete(&server.url(), &client, "", "testvnet", "10.0.0.0-24").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_cidr_to_subnet_id() {
        assert_eq!(cidr_to_subnet_id("10.0.0.0/24"), "10.0.0.0-24");
        assert_eq!(cidr_to_subnet_id("192.168.1.0/16"), "192.168.1.0-16");
    }
}
