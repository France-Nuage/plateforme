use crate::api::pagination::PaginableOperationQuery;
use crate::api::{ApiOperationQuery, ApiResponse};
use crate::models::Instance;

#[derive(Clone)]
pub struct ComputeInstanceIndexQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ApiOperationQuery for ComputeInstanceIndexQuery {}

impl PaginableOperationQuery for ComputeInstanceIndexQuery {
    fn set_page_id(&mut self, page_id: Option<u32>) {
        self.page = page_id;
    }
}

impl Default for ComputeInstanceIndexQuery {
    fn default() -> Self {
        ComputeInstanceIndexQuery {
            page: None,
            per_page: Some(10),
        }
    }
}

/// Perform an HTTP call to the API on the compute instances index endpoint
pub async fn list_compute_instances(
    client: &reqwest::Client,
    query: ComputeInstanceIndexQuery,
) -> Result<ApiResponse<Vec<Instance>>, reqwest::Error> {
    let response = client
        .get(format!("{}/api/v1/compute/instances", query.base_url()))
        .header("Authorization", format!("Bearer {}", query.access_token()))
        .query(&[("page", query.page)])
        .query(&[("perPage", query.per_page)])
        .send()
        .await?;

    response.json().await
}
