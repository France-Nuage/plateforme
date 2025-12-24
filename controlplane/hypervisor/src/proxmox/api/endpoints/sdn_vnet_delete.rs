//! SDN VNet deletion endpoint for Proxmox VE.
//!
//! Deletes a VNet from the Proxmox SDN configuration.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Deletes an SDN VNet from Proxmox.
///
/// API: DELETE /cluster/sdn/vnets/{vnet}
pub async fn sdn_vnet_delete(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    vnet: &str,
) -> Result<ApiResponse<Option<String>>, Error> {
    client
        .delete(format!("{}/api2/json/cluster/sdn/vnets/{}", api_url, vnet))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSDNVNetDeleteMock {
        fn with_sdn_vnet_delete(self) -> Self;
    }

    impl WithSDNVNetDeleteMock for MockServer {
        fn with_sdn_vnet_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api2/json/cluster/sdn/vnets/.+$".to_string()),
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
    use super::mock::WithSDNVNetDeleteMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_vnet_delete() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_vnet_delete();
        let result = sdn_vnet_delete(&server.url(), &client, "", "vnet-test").await;

        assert!(result.is_ok());
    }
}
