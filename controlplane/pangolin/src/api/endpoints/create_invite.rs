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
    /// Role ID to assign to the user (must be a number for the integration API).
    pub role_id: i64,
    /// Whether to send an email notification.
    pub send_email: bool,
    /// Validity duration for the invitation in hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_hours: Option<i64>,
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
/// * `api_url` - The Pangolin Integration API base URL (port 3003)
/// * `client` - HTTP client
/// * `api_key` - Pangolin API key for authorization
/// * `org_id` - The organization slug/ID in Pangolin
/// * `email` - Email address of the user to invite
/// * `role_id` - Role ID to assign (numeric)
/// * `send_email` - Whether Pangolin should send the invitation email
/// * `valid_hours` - Optional validity duration in hours
///
/// # Returns
/// The invite response containing the invite ID and token.
pub async fn create_invite(
    api_url: &str,
    client: &reqwest::Client,
    api_key: &str,
    org_id: &str,
    email: &str,
    role_id: i64,
    send_email: bool,
    valid_hours: Option<i64>,
) -> Result<CreateInviteResponse, Error> {
    let request = CreateInviteRequest {
        email: email.to_string(),
        role_id,
        send_email,
        valid_hours,
    };

    // Integration API uses /v1/ prefix (not /api/v1/) and bypasses CSRF protection
    client
        .post(format!("{}/v1/org/{}/create-invite", api_url, org_id))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .to_json()
        .await
}
