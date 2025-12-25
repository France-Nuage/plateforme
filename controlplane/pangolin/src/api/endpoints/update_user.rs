//! Update user endpoint.
//!
//! Updates a user's role or status within an organization in Pangolin.

use serde::Serialize;

use crate::api::Error;
use crate::api::api_response::ApiResponseExt;

/// Request body for updating a user.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    /// New role ID for the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    /// Whether the user is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// Updates a user in an organization in Pangolin.
///
/// # Arguments
/// * `api_url` - The Pangolin API base URL
/// * `client` - HTTP client
/// * `api_key` - Pangolin API key for authorization
/// * `org_id` - The organization slug/ID in Pangolin
/// * `user_id` - The ID of the user to update
/// * `role_id` - Optional new role ID
/// * `disabled` - Optional disabled state
pub async fn update_user(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    org_id: &str,
    user_id: &str,
    role_id: Option<&str>,
    disabled: Option<bool>,
) -> Result<(), Error> {
    let request = UpdateUserRequest {
        role_id: role_id.map(|s| s.to_string()),
        disabled,
    };

    client
        .patch(format!(
            "{}/api/v1/org/{}/user/{}",
            api_url, org_id, user_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .to_empty()
        .await
}
