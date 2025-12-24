//! SDN Apply endpoint for Proxmox VE.
//!
//! Applies/reloads the SDN configuration across the cluster.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Applies the SDN configuration in Proxmox.
///
/// This must be called after making changes to zones, VNets, or subnets
/// for the changes to take effect.
///
/// API: PUT /cluster/sdn
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

    pub trait WithSDNApplyMock {
        fn with_sdn_apply(self) -> Self;
    }

    impl WithSDNApplyMock for MockServer {
        fn with_sdn_apply(mut self) -> Self {
            let mock = self
                .server
                .mock("PUT", "/api2/json/cluster/sdn")
                .with_body(r#"{"data":"UPID:pve-node1:0021B1A0:02328822:67CC7B44:sdnapply::root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithSDNApplyMock;
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
