//! SDN configuration apply for Proxmox.
//!
//! Applies pending SDN changes to all cluster nodes. Must be called after creating/deleting
//! zones, VNets, or subnets.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Applies the SDN configuration to all nodes in the Proxmox cluster.
///
/// This must be called after any SDN modification (zone, vnet, subnet creation
/// or deletion) to deploy the configuration across all cluster nodes.
pub async fn sdn_apply(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
) -> Result<ApiResponse<String>, Error> {
    client
        .put(format!("{}/api2/json/cluster/sdn", api_url))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnApplyMock {
        fn with_sdn_apply(self) -> Self;
    }

    impl WithSdnApplyMock for MockServer {
        fn with_sdn_apply(mut self) -> Self {
            let mock = self
                .server
                .mock("PUT", "/api2/json/cluster/sdn")
                .with_body(r#"{"data":"UPID:pve-node1:0021B19E:02328820:67CC7B42:sdnapply::root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithSdnApplyMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_apply() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_apply();
        let result = sdn_apply(&server.url(), &client, "").await;
        assert!(result.is_ok());
    }
}
