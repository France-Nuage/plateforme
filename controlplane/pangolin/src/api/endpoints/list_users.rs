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
/// * `api_url` - The Pangolin Integration API base URL (port 3003)
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
    // Integration API uses /v1/ prefix (not /api/v1/) and bypasses CSRF protection
    client
        .get(format!("{}/v1/org/{}/users", api_url, org_id))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .to_json()
        .await
}
