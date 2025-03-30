use crate::{InstanceConfig, InstanceService, InstanceStatus, Problem, v1::InstanceInfo};

pub struct ProxmoxInstanceService {
    pub api_url: String,
    pub client: reqwest::Client,
    pub id: u32,
}

impl InstanceService for ProxmoxInstanceService {
    async fn list(&self) -> Result<Vec<InstanceInfo>, Problem> {
        let response = proxmox::cluster_resources_list(&self.api_url, &self.client)
            .await
            .expect("could not fetch vms")
            .data;
        Ok(response)
    }

    /// Creates the instance.
    async fn create(&self, options: InstanceConfig) -> Result<(), Problem> {
        proxmox::vm_create(&self.api_url, &self.client, "pve-node1", &options.into())
            .await
            .expect("could not create a new proxmox vm");

        Ok(())
    }

    /// Deletes the instance.
    async fn delete(&self) -> Result<(), Problem> {
        proxmox::vm_delete(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not delete proxmox vm");
        Ok(())
    }

    /// Starts the instance.
    async fn start(&self) -> Result<(), Problem> {
        proxmox::vm_status_start(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not start proxmox vm");
        Ok(())
    }

    /// Gets the instance status.
    async fn status(&self) -> Result<InstanceStatus, Problem> {
        let result = proxmox::vm_status_read(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not get proxmox vm status");
        Ok(result.data.status.into())
    }

    /// Stops the instance.
    async fn stop(&self) -> Result<(), Problem> {
        proxmox::vm_status_stop(&self.api_url, &self.client, "pve-node1", self.id)
            .await
            .expect("could not stop proxmox vm");
        Ok(())
    }
}

struct ProxmoxInstanceConfig {
    id: String,
    name: String,
}

impl From<InstanceConfig> for proxmox::VMConfig {
    fn from(value: InstanceConfig) -> Self {
        proxmox::VMConfig {
            vmid: value
                .id
                .parse::<u32>()
                .expect("could notp;arse the given id to a u32"),
            name: Some(value.name),
            ..Default::default()
        }
    }
}

impl From<proxmox::VMStatus> for InstanceStatus {
    fn from(value: proxmox::VMStatus) -> Self {
        match value {
            proxmox::VMStatus::Running => InstanceStatus::Running,
            proxmox::VMStatus::Stopped => InstanceStatus::Stopped,
        }
    }
}
