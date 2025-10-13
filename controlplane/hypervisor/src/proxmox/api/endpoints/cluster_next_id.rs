use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};

pub async fn cluster_next_id(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
) -> Result<ApiResponse<u32>, crate::proxmox::api::Error> {
    client
        .get(format!("{}/api2/json/cluster/nextid", api_url))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response::<String>()
        .await
        .map(|response| ApiResponse {
            data: response
                .data
                .parse::<u32>()
                .expect("expected proxmox api to return a number"),
        })
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithClusterNextId {
        fn with_cluster_next_id(self) -> Self;
    }

    impl WithClusterNextId for MockServer {
        fn with_cluster_next_id(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api2/json/cluster/nextid$".to_string()),
                )
                .with_body(r#"{"data": "100"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithClusterNextId;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_cluster_resource_list() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_cluster_next_id();
        let result = cluster_next_id(&server.url(), &client, "").await;

        assert!(result.is_ok());
        let next_id = result.unwrap().data;
        assert_eq!(next_id, 100);
    }
}
