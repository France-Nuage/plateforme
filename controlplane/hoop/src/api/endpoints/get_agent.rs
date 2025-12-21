//! Get agent endpoint.
//!
//! Retrieves a Hoop agent by name or ID.

use serde::Deserialize;

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Response from getting an agent.
#[derive(Debug, Deserialize)]
pub struct GetAgentResponse {
    /// The agent's unique ID (UUID).
    pub id: String,
    /// The agent's name.
    pub name: String,
    /// The agent's status (CONNECTED or DISCONNECTED).
    pub status: String,
}

/// Gets a Hoop agent by name or ID.
///
/// # Arguments
/// * `api_url` - The Hoop API base URL (e.g., `https://bastion.ssh.france-nuage.fr`)
/// * `client` - HTTP client
/// * `api_key` - Hoop API key for authorization
/// * `name_or_id` - Name or ID of the agent to retrieve
///
/// # Returns
/// The agent details including its UUID.
pub async fn get_agent(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    name_or_id: &str,
) -> Result<GetAgentResponse, Error> {
    client
        .get(format!("{}/api/agents/{}", api_url, name_or_id))
        .header("Api-Key", api_key)
        .send()
        .await
        .to_json()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithGetAgentMock {
        fn with_get_agent(self) -> Self;
    }

    impl WithGetAgentMock for MockServer {
        fn with_get_agent(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api/agents/.+$".to_string()),
                )
                .with_status(200)
                .with_body(r#"{"id":"550e8400-e29b-41d4-a716-446655440000","name":"test-instance","status":"DISCONNECTED"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::mock::WithGetAgentMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_get_agent() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_get_agent();

        let result = get_agent(&server.url(), &client, "test-api-key", "test-instance").await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(agent.name, "test-instance");
    }
}
