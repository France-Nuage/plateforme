//! Create agent endpoint.
//!
//! Creates a new Hoop agent and returns its token for VM configuration.

use serde::{Deserialize, Serialize};

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Request body for creating an agent.
#[derive(Debug, Serialize)]
pub struct CreateAgentRequest {
    /// Name of the agent (typically the instance name).
    pub name: String,
    /// Agent mode: "standard" or "embedded".
    pub mode: String,
}

/// Response from creating an agent.
#[derive(Debug, Deserialize)]
pub struct CreateAgentResponse {
    /// The agent token in DSN format.
    /// Example: "grpc://name:token@gateway:port?mode=standard"
    pub token: String,
}

/// Creates a new Hoop agent.
///
/// # Arguments
/// * `api_url` - The Hoop API base URL (e.g., `https://bastion.ssh.france-nuage.fr`)
/// * `client` - HTTP client
/// * `api_key` - Hoop API key for authorization
/// * `name` - Name for the agent (typically the instance name)
///
/// # Returns
/// The agent token (DSN format) to be used in cloud-init configuration.
pub async fn create_agent(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    name: &str,
) -> Result<String, Error> {
    let request = CreateAgentRequest {
        name: name.to_string(),
        mode: "standard".to_string(),
    };

    let response: CreateAgentResponse = client
        .post(format!("{}/api/agents", api_url))
        .header("Api-Key", api_key)
        .json(&request)
        .send()
        .await
        .to_json()
        .await?;

    Ok(response.token)
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithCreateAgentMock {
        fn with_create_agent(self) -> Self;
    }

    impl WithCreateAgentMock for MockServer {
        fn with_create_agent(mut self) -> Self {
            let mock = self
                .server
                .mock("POST", "/api/agents")
                .with_status(201)
                .with_body(r#"{"token":"grpc://test-instance:abc123token@gateway.hoop.dev:443?mode=standard"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::mock::WithCreateAgentMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_create_agent() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_create_agent();

        let result = create_agent(&server.url(), &client, "test-api-key", "test-instance").await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(token.contains("test-instance"));
        assert!(token.contains("grpc://"));
    }
}
