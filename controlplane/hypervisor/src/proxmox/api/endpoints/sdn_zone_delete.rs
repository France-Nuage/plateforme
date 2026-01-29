use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Deletes an SDN zone from the Proxmox cluster.
///
/// The zone must not have any VNets attached before deletion.
pub async fn sdn_zone_delete(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    zone_id: &str,
) -> Result<ApiResponse<()>, Error> {
    client
        .delete(format!("{}/api2/json/cluster/sdn/zones/{}", api_url, zone_id))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSdnZoneDeleteMock {
        fn with_sdn_zone_delete(self) -> Self;
    }

    impl WithSdnZoneDeleteMock for MockServer {
        fn with_sdn_zone_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api2/json/cluster/sdn/zones/.*$".to_string()),
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
    use super::mock::WithSdnZoneDeleteMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_zone_delete() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_zone_delete();
        let result = sdn_zone_delete(&server.url(), &client, "", "testzone").await;
        assert!(result.is_ok());
    }
}
