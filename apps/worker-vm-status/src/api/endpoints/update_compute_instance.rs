use serde::Serialize;

use crate::{
    api::ApiOperationQuery,
    models::{Instance, InstanceStatus},
};

#[derive(Debug)]
pub struct UpdateComputeInstanceQuery {
    pub instance_id: String,
    pub status: InstanceStatus,
}

#[derive(Debug, Serialize)]
pub struct UpdateComputeInstanceBody {
    pub status: InstanceStatus,
}

impl ApiOperationQuery for UpdateComputeInstanceQuery {}

pub async fn update_compute_instance(
    client: &reqwest::Client,
    query: UpdateComputeInstanceQuery,
) -> Result<Instance, reqwest::Error> {
    let body = UpdateComputeInstanceBody {
        status: query.status.clone(),
    };

    client
        .patch(format!(
            "{}/api/v1/compute/instances/{}",
            query.base_url(),
            query.instance_id
        ))
        .header("Authorization", format!("Bearer {}", query.access_token()))
        .json(&body)
        .send()
        .await?
        .json()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_compute_instance_works() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock(
                "PATCH",
                "/api/v1/compute/instances/00000000-0000-0000-0000-000000000000",
            )
            .with_status(200)
            .with_body(r#"{"id":"079ce056-f5a0-4b87-bec1-d34b3c14acb5","name":"Plopzfe","nodeId":"00000000-0000-0000-0000-000000000007","bootDiskId":null,"createdAt":"2024-12-24T13:19:37.061+00:00","updatedAt":"2025-02-02T12:37:18.406+00:00","pveVmId":"105","projectId":null,"status":"STOPPED"}"#)
            .create();

        std::env::set_var("FRANCE_NUAGE_API_URL", server.url());
        std::env::set_var("FRANCE_NUAGE_API_TOKEN", "fak3_acc3ss_t0k3n");

        let client = reqwest::Client::new();
        let query = UpdateComputeInstanceQuery {
            instance_id: String::from("00000000-0000-0000-0000-000000000000"),
            status: InstanceStatus::Stopped,
        };

        let result = update_compute_instance(&client, query).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.status, InstanceStatus::Stopped);
    }
}
