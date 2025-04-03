use hypervisor_connector::{
    InstanceConfig, InstanceInfo, InstanceService, InstanceStatus, Problem,
};

pub struct ProxmoxInstanceService {
    pub api_url: String,
    pub client: reqwest::Client,
    pub id: u32,
}

impl InstanceService for ProxmoxInstanceService {
    async fn list(&self) -> Result<Vec<InstanceInfo>, Problem> {
        let response = crate::proxmox_api::cluster_resources_list(&self.api_url, &self.client)
            .await
            .expect("could not fetch vms")
            .data;
        Ok(response.into_iter().map(Into::into).collect())
    }

    /// Creates the instance.
    async fn create(&self, options: InstanceConfig) -> Result<(), Problem> {
        crate::proxmox_api::vm_create(&self.api_url, &self.client, "pve-node1", &options.into())
            .await
            .expect("could not create a new proxmox vm");

        Ok(())
    }

    /// Deletes the instance.
    async fn delete(&self) -> Result<(), Problem> {
        crate::proxmox_api::vm_delete(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not delete proxmox vm");
        Ok(())
    }

    /// Starts the instance.
    async fn start(&self) -> Result<(), Problem> {
        crate::proxmox_api::vm_status_start(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not start proxmox vm");
        Ok(())
    }

    /// Gets the instance status.
    async fn status(&self) -> Result<InstanceStatus, Problem> {
        let result =
            crate::proxmox_api::vm_status_read(&self.api_url, &self.client, "pve-node1", self.id)
                .await
                .expect("could not get proxmox vm status");
        Ok(result.data.status.into())
    }

    /// Stops the instance.
    async fn stop(&self) -> Result<(), Problem> {
        crate::proxmox_api::vm_status_stop(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not stop proxmox vm");
        Ok(())
    }
}
