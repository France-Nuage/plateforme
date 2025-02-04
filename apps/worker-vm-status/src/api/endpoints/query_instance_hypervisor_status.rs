use serde::Deserialize;

use crate::models::{Instance, InstanceStatus};

use crate::api::ApiOperationQuery;

pub struct InstanceHypervisorStatusQuery {
    node_id: String,
    pve_vm_id: String,
}

impl ApiOperationQuery for InstanceHypervisorStatusQuery {}

impl InstanceHypervisorStatusQuery {
    pub fn from_instance(instance: &Instance) -> Self {
        InstanceHypervisorStatusQuery {
            node_id: instance.node_id.clone(),
            pve_vm_id: instance.pve_vm_id.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QemuStatusApiResponse {
    pub status: InstanceStatus,
}

pub async fn get_instance_hypervisor_status(
    client: &reqwest::Client,
    query: InstanceHypervisorStatusQuery,
) -> Result<QemuStatusApiResponse, reqwest::Error> {
    client
        .get(format!(
            "{}/api/internal/hypervisor/nodes/{}/qemu/{}/status/current",
            query.base_url(),
            query.node_id,
            query.pve_vm_id,
        ))
        .header("Authorization", format!("Bearer {}", query.access_token()))
        .send()
        .await?
        .json()
        .await
}

#[cfg(test)]
mod tests {
    use crate::models::InstanceStatus;

    use super::*;

    #[tokio::test]
    async fn test_get_instance_hypervisor_status_works() {
        let instance = Instance {
            id: String::from("00000000-0000-0000-0000-000000000000"),
            node_id: String::from("00000000-0000-0000-0000-000000000007"),
            pve_vm_id: String::from("105"),
            status: InstanceStatus::Stopping,
        };

        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/api/internal/hypervisor/nodes/00000000-0000-0000-0000-000000000007/qemu/105/status/current")
            .with_status(200)
            .with_body(r#"{"status":"STOPPED"}"#)
            .create();

        std::env::set_var("FRANCE_NUAGE_API_URL", server.url());
        std::env::set_var("FRANCE_NUAGE_API_TOKEN", "fak3_acc3ss_t0k3n");

        let client = reqwest::Client::new();
        let query = InstanceHypervisorStatusQuery::from_instance(&instance);

        let result = get_instance_hypervisor_status(&client, query).await;
        assert!(result.is_ok());

        let status_response = result.unwrap();
        assert_eq!(status_response.status, InstanceStatus::Stopped);
    }
}
