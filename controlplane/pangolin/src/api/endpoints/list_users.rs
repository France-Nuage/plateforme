//! List users endpoint.
//!
//! Lists all users in an organization in Pangolin.

use serde::Deserialize;

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// A user within an organization.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgUser {
    /// The user's ID in Pangolin.
    pub id: String,
    /// The user's email address.
    pub email: String,
    /// The user's role ID.
    pub role_id: String,
    /// Whether the user is disabled.
    pub disabled: bool,
    /// When the user joined the organization.
    pub joined_at: String,
}

/// Response from listing users.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersResponse {
    /// List of users in the organization.
    pub users: Vec<OrgUser>,
    /// Total count of users.
    pub total: i64,
}

/// Lists all users in an organization in Pangolin.
///
/// # Arguments
/// * `api_url` - The Pangolin API base URL
/// * `client` - HTTP client
/// * `api_key` - Pangolin API key for authorization
/// * `org_id` - The organization slug/ID in Pangolin
///
/// # Returns
/// The list of users in the organization.
pub async fn list_users(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    org_id: &str,
) -> Result<ListUsersResponse, Error> {
    client
        .get(format!("{}/api/v1/org/{}/users", api_url, org_id))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .to_json()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithListUsersMock {
        fn with_list_users(self) -> Self;
    }

    impl WithListUsersMock for MockServer {
        fn with_list_users(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api/v1/org/[^/]+/users$".to_string()),
                )
                .with_status(200)
                .with_body(
                    r#"{"users":[{"id":"user_123","email":"user@example.com","roleId":"role_member","disabled":false,"joinedAt":"2024-01-01T00:00:00Z"}],"total":1}"#,
                )
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::mock::WithListUsersMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_list_users() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_list_users();

        let result = list_users(&server.url(), &client, "test-api-key", "test-org").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total, 1);
        assert_eq!(response.users.len(), 1);
        assert_eq!(response.users[0].email, "user@example.com");
    }
}
