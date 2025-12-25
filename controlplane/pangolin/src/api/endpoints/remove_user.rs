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
