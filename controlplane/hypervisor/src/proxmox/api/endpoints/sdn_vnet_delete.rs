use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Deletes an SDN VNet from the Proxmox cluster.
///
/// The VNet must not have any subnets attached before deletion.
pub async fn sdn_vnet_delete(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    vnet_id: &str,
) -> Result<ApiResponse<()>, Error> {
    client
        .delete(format!("{}/api2/json/cluster/sdn/vnets/{}", api_url, vnet_id))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnVnetDeleteMock {
        fn with_sdn_vnet_delete(self) -> Self;
    }

    impl WithSdnVnetDeleteMock for MockServer {
        fn with_sdn_vnet_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api2/json/cluster/sdn/vnets/.*$".to_string()),
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
    use super::mock::WithSdnVnetDeleteMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_vnet_delete() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_vnet_delete();
        let result = sdn_vnet_delete(&server.url(), &client, "", "testvnet").await;
        assert!(result.is_ok());
    }
}
