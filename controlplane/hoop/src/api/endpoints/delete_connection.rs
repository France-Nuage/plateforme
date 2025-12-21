//! Delete connection endpoint.
//!
//! Deletes a Hoop connection by name.

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Deletes a Hoop connection.
///
/// # Arguments
/// * `api_url` - The Hoop API base URL (e.g., `https://bastion.ssh.france-nuage.fr`)
/// * `client` - HTTP client
/// * `api_key` - Hoop API key for authorization
/// * `name` - Name of the connection to delete
pub async fn delete_connection(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    name: &str,
) -> Result<(), Error> {
    client
        .delete(format!("{}/api/connections/{}", api_url, name))
        .header("Api-Key", api_key)
        .send()
        .await
        .to_empty()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithDeleteConnectionMock {
        fn with_delete_connection(self) -> Self;
    }

    impl WithDeleteConnectionMock for MockServer {
        fn with_delete_connection(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api/connections/.+$".to_string()),
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
    use super::mock::WithDeleteConnectionMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_delete_connection() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_delete_connection();

        let result =
            delete_connection(&server.url(), &client, "test-api-key", "test-instance").await;

        assert!(result.is_ok());
    }
}
