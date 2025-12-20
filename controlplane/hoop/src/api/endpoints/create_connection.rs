//! Create connection endpoint.
//!
//! Creates a new SSH connection linking an agent to SSH credentials.

use serde::Serialize;

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Secret configuration for SSH connection.
#[derive(Debug, Serialize)]
pub struct ConnectionSecret {
    /// SSH host (typically "127.0.0.1" since agent is on the VM).
    #[serde(rename = "envvar:HOST")]
    pub host: String,
    /// SSH user.
    #[serde(rename = "envvar:USER")]
    pub user: String,
    /// SSH private key (base64 encoded, PKCS#8 format).
    #[serde(rename = "filesystem:SSH_PRIVATE_KEY")]
    pub private_key: String,
}

/// Request body for creating a connection.
#[derive(Debug, Serialize)]
pub struct CreateConnectionRequest {
    /// Name of the connection (typically the instance name).
    pub name: String,
    /// Type of connection.
    #[serde(rename = "type")]
    pub connection_type: String,
    /// Subtype of connection.
    pub subtype: String,
    /// Agent ID/name to use for this connection.
    pub agent_id: String,
    /// SSH secrets (host, user, private key).
    pub secret: ConnectionSecret,
    /// Enable connect access mode.
    pub access_mode_connect: bool,
    /// Enable exec access mode.
    pub access_mode_exec: bool,
    /// Enable runbooks access mode.
    pub access_mode_runbooks: bool,
}

/// Creates a new SSH connection in Hoop.
///
/// # Arguments
/// * `api_url` - The Hoop API base URL (e.g., `https://bastion.ssh.france-nuage.fr`)
/// * `client` - HTTP client
/// * `api_key` - Hoop API key for authorization
/// * `name` - Name for the connection (typically the instance name)
/// * `agent_id` - Agent ID/name to associate with this connection
/// * `user` - SSH user on the VM
/// * `private_key` - SSH private key (base64 encoded, PKCS#8 format)
pub async fn create_connection(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    name: &str,
    agent_id: &str,
    user: &str,
    private_key: &str,
) -> Result<(), Error> {
    let request = CreateConnectionRequest {
        name: name.to_string(),
        connection_type: "application".to_string(),
        subtype: "ssh".to_string(),
        agent_id: agent_id.to_string(),
        secret: ConnectionSecret {
            host: "127.0.0.1".to_string(),
            user: user.to_string(),
            private_key: private_key.to_string(),
        },
        access_mode_connect: true,
        access_mode_exec: true,
        access_mode_runbooks: false,
    };

    client
        .post(format!("{}/api/connections", api_url))
        .header("Api-Key", api_key)
        .json(&request)
        .send()
        .await
        .to_empty()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithCreateConnectionMock {
        fn with_create_connection(self) -> Self;
    }

    impl WithCreateConnectionMock for MockServer {
        fn with_create_connection(mut self) -> Self {
            let mock = self
                .server
                .mock("POST", "/api/connections")
                .with_status(201)
                .with_body(r#"{"id":"conn-123","name":"test-instance","status":"offline"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithCreateConnectionMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_create_connection() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_create_connection();

        let result = create_connection(
            &server.url(),
            &client,
            "test-api-key",
            "test-instance",
            "test-instance",
            "francenuage",
            "base64-encoded-private-key",
        )
        .await;

        assert!(result.is_ok());
    }
}
