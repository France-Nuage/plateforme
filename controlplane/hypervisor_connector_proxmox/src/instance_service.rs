use crate::proxmox_api::{helpers, vm_create::VMConfig};
use hypervisor_connector::{
    InstanceConfig, InstanceInfo, InstanceService, InstanceStatus, Problem,
};
pub struct ProxmoxInstanceService {
    pub api_url: String,
    pub client: reqwest::Client,
    pub authorization: String,
    pub id: u32,
}

impl InstanceService for ProxmoxInstanceService {
    async fn list(&self) -> Result<Vec<InstanceInfo>, Problem> {
        let response = crate::proxmox_api::cluster_resources_list(
            &self.api_url,
            &self.client,
            &self.authorization,
        )
        .await?
        .data;
        Ok(response.into_iter().map(Into::into).collect())
    }

    /// Creates the instance.
    async fn create(&self, options: InstanceConfig) -> Result<String, Problem> {
        let next_id =
            crate::proxmox_api::cluster_next_id(&self.api_url, &self.client, &self.authorization)
                .await?
                .data;

        // TODO: use the (not yet developped) scheduler to select a node
        let node_id = "pvedev01-dc03";
        let vm_config = VMConfig::from_instance_config(options, next_id);

        let task_id = crate::proxmox_api::vm_create(
            &self.api_url,
            &self.client,
            &self.authorization,
            node_id,
            &vm_config,
        )
        .await?
        .data;

        crate::proxmox_api::helpers::wait_for_task_completion(
            &self.api_url,
            &self.client,
            &self.authorization,
            node_id,
            &task_id,
        )
        .await
        .map(|result| result.id)
        .map_err(Into::into)
    }

    /// Deletes the instance.
    async fn delete(&self) -> Result<(), Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            self.id,
        )
        .await?;

        crate::proxmox_api::vm_delete(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            self.id,
        )
        .await?;
        Ok(())
    }

    /// Starts the instance.
    async fn start(&self) -> Result<(), Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            self.id,
        )
        .await?;

        crate::proxmox_api::vm_status_start(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            self.id,
        )
        .await?;
        Ok(())
    }

    /// Gets the instance status.
    async fn status(&self) -> Result<InstanceStatus, Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            self.id,
        )
        .await?;

        let result = crate::proxmox_api::vm_status_read(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            self.id,
        )
        .await?;
        Ok(result.data.status.into())
    }

    /// Stops the instance.
    async fn stop(&self) -> Result<(), Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            self.id,
        )
        .await?;

        crate::proxmox_api::vm_status_stop(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            self.id,
        )
        .await?;
        Ok(())
    }
}
