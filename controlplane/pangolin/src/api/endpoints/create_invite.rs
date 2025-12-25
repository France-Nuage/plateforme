//! Create invite endpoint.
//!
//! Creates a new invitation for a user to join an organization in Pangolin.

use serde::{Deserialize, Serialize};

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Request body for creating an invite.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteRequest {
    /// Email address of the user to invite.
    pub email: String,
    /// Role ID to assign to the user.
    pub role_id: String,
    /// Whether to send an email notification.
    pub send_email: bool,
    /// Validity duration for the invitation (e.g., "24h", "7d").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for_hours: Option<i64>,
}

/// Response from creating an invite.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteResponse {
    /// The generated invite ID.
    pub invite_id: String,
    /// The invite token (for the invite URL).
    pub token: String,
    /// Expiration timestamp.
    pub expires_at: String,
}

/// Creates a new invitation in Pangolin.
///
/// # Arguments
/// * `api_url` - The Pangolin API base URL
/// * `client` - HTTP client
/// * `api_key` - Pangolin API key for authorization
/// * `org_id` - The organization slug/ID in Pangolin
/// * `email` - Email address of the user to invite
/// * `role_id` - Role ID to assign
/// * `send_email` - Whether Pangolin should send the invitation email
/// * `valid_for_hours` - Optional validity duration in hours
///
/// # Returns
/// The invite response containing the invite ID and token.
pub async fn create_invite(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    org_id: &str,
    email: &str,
    role_id: &str,
    send_email: bool,
    valid_for_hours: Option<i64>,
) -> Result<CreateInviteResponse, Error> {
    let request = CreateInviteRequest {
        email: email.to_string(),
        role_id: role_id.to_string(),
        send_email,
        valid_for_hours,
    };

    client
        .post(format!("{}/api/v1/org/{}/create-invite", api_url, org_id))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .to_json()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithCreateInviteMock {
        fn with_create_invite(self) -> Self;
    }

    impl WithCreateInviteMock for MockServer {
        fn with_create_invite(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(r"^/api/v1/org/[^/]+/create-invite$".to_string()),
                )
                .with_status(201)
                .with_body(
                    r#"{"inviteId":"inv_123","token":"abc123token","expiresAt":"2024-12-25T00:00:00Z"}"#,
                )
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::mock::WithCreateInviteMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_create_invite() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_create_invite();

        let result = create_invite(
            &server.url(),
            &client,
            "test-api-key",
            "test-org",
            "user@example.com",
            "role_member",
            true,
            Some(24),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.invite_id, "inv_123");
        assert_eq!(response.token, "abc123token");
    }
}
