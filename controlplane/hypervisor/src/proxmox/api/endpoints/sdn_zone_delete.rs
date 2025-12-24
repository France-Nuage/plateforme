//! SDN Zone deletion endpoint for Proxmox VE.
//!
//! Deletes a zone from the Proxmox SDN configuration.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

/// Deletes an SDN zone from Proxmox.
///
/// API: DELETE /cluster/sdn/zones/{zone}
pub async fn sdn_zone_delete(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    zone: &str,
) -> Result<ApiResponse<Option<String>>, Error> {
    client
        .delete(format!("{}/api2/json/cluster/sdn/zones/{}", api_url, zone))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithSDNZoneDeleteMock {
        fn with_sdn_zone_delete(self) -> Self;
    }

    impl WithSDNZoneDeleteMock for MockServer {
        fn with_sdn_zone_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api2/json/cluster/sdn/zones/.+$".to_string()),
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
    use super::mock::WithSDNZoneDeleteMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_sdn_zone_delete() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_sdn_zone_delete();
        let result = sdn_zone_delete(&server.url(), &client, "", "zone-test").await;

        assert!(result.is_ok());
    }
}
