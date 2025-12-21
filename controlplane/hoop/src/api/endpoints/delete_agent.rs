//! Delete agent endpoint.
//!
//! Deletes a Hoop agent by name or ID.

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Deletes a Hoop agent.
///
/// # Arguments
/// * `api_url` - The Hoop API base URL (e.g., `https://bastion.ssh.france-nuage.fr`)
/// * `client` - HTTP client
/// * `api_key` - Hoop API key for authorization
/// * `name` - Name or ID of the agent to delete
pub async fn delete_agent(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    name: &str,
) -> Result<(), Error> {
    client
        .delete(format!("{}/api/agents/{}", api_url, name))
        .header("Api-Key", api_key)
        .send()
        .await
        .to_empty()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithDeleteAgentMock {
        fn with_delete_agent(self) -> Self;
    }

    impl WithDeleteAgentMock for MockServer {
        fn with_delete_agent(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api/agents/.+$".to_string()),
                )
                .with_status(204)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::mock::WithDeleteAgentMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_delete_agent() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_delete_agent();

        let result = delete_agent(&server.url(), &client, "test-api-key", "test-instance").await;

        assert!(result.is_ok());
    }
}
