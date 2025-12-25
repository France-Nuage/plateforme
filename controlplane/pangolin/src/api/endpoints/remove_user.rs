//! Remove user endpoint.
//!
//! Removes a user from an organization in Pangolin.

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Removes a user from an organization in Pangolin.
///
/// # Arguments
/// * `api_url` - The Pangolin API base URL
/// * `client` - HTTP client
/// * `api_key` - Pangolin API key for authorization
/// * `org_id` - The organization slug/ID in Pangolin
/// * `user_id` - The ID of the user to remove
pub async fn remove_user(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    org_id: &str,
    user_id: &str,
) -> Result<(), Error> {
    client
        .delete(format!(
            "{}/api/v1/org/{}/user/{}",
            api_url, org_id, user_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .to_empty()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithRemoveUserMock {
        fn with_remove_user(self) -> Self;
    }

    impl WithRemoveUserMock for MockServer {
        fn with_remove_user(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api/v1/org/[^/]+/user/[^/]+$".to_string()),
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
    use super::mock::WithRemoveUserMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_remove_user() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_remove_user();

        let result = remove_user(
            &server.url(),
            &client,
            "test-api-key",
            "test-org",
            "user_123",
        )
        .await;

        assert!(result.is_ok());
    }
}
