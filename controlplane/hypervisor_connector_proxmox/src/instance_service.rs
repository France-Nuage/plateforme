use std::net::Ipv4Addr;

use crate::proxmox_api::{self, helpers, vm_clone::VMCloneOptions, vm_create::VMConfig};
use hypervisor_connector::{
    InstanceConfig, InstanceInfo, InstanceService, InstanceStatus, Problem,
};
pub struct ProxmoxInstanceService {
    pub api_url: String,
    pub client: reqwest::Client,
    pub authorization: String,
}

impl InstanceService for ProxmoxInstanceService {
    async fn list(&self) -> Result<Vec<InstanceInfo>, Problem> {
        let response = crate::proxmox_api::cluster_resources_list(
            &self.api_url,
            &self.client,
            &self.authorization,
            "vm",
        )
        .await?
        .data;
        response
            .into_iter()
            .map(|resource| resource.try_into().map_err(Into::into))
            .collect()
    }

    /// Clones the instance.
    async fn clone(&self, id: &str) -> Result<String, Problem> {
        let id = id
            .parse::<u32>()
            .map_err(|_| Problem::MalformedVmId(id.to_owned()))?;

        let next_id =
            crate::proxmox_api::cluster_next_id(&self.api_url, &self.client, &self.authorization)
                .await?
                .data;

        let node_id =
            helpers::get_vm_execution_node(&self.api_url, &self.client, &self.authorization, id)
                .await?;

        let options = VMCloneOptions {
            newid: next_id,
            full: true,
        };

        let task = crate::proxmox_api::vm_clone(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id,
            &options,
        )
        .await?
        .data;

        proxmox_api::helpers::wait_for_task_completion(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &task,
        )
        .await?;

        Ok(next_id.to_string())
    }

    /// Creates the instance.
    async fn create(&self, options: InstanceConfig) -> Result<String, Problem> {
        let next_id =
            crate::proxmox_api::cluster_next_id(&self.api_url, &self.client, &self.authorization)
                .await?
                .data;

        let node_id = crate::proxmox_api::cluster_resources_list(
            &self.api_url,
            &self.client,
            &self.authorization,
            "node",
        )
        .await?
        .data
        .first()
        .ok_or_else(|| crate::proxmox_api::Problem::NoNodesAvailable)?
        .node
        .clone()
        .expect("node should be defined for resource of type node");

        let vm_config = VMConfig::from_instance_config(options, next_id);

        let task_id = crate::proxmox_api::vm_create(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &vm_config,
        )
        .await?
        .data;

        crate::proxmox_api::helpers::wait_for_task_completion(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &task_id,
        )
        .await
        .map(|result| result.id)
        .map_err(Into::into)
    }

    /// Gets the instance ip address.
    async fn get_ip_address(&self, id: &str) -> Result<Option<Ipv4Addr>, Problem> {
        // Parse the id to a u32
        let id = id
            .parse::<u32>()
            .map_err(|_| Problem::MalformedVmId(id.to_owned()))?;

        // Get the VM execution node
        let node_id =
            helpers::get_vm_execution_node(&self.api_url, &self.client, &self.authorization, id)
                .await?;

        // Get the VM network interfaces
        let ip = crate::proxmox_api::vm_config_read(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id,
        )
        .await?
        .data
        .ipconfig0
        .map(|config| config.ip);

        Ok(ip)
    }

    /// Deletes the instance.
    async fn delete(&self, id: &str) -> Result<(), Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let task = crate::proxmox_api::vm_delete(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?
        .data;

        proxmox_api::helpers::wait_for_task_completion(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &task,
        )
        .await?;

        Ok(())
    }

    /// Starts the instance.
    async fn start(&self, id: &str) -> Result<(), Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let task = crate::proxmox_api::vm_status_start(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?
        .data;

        proxmox_api::helpers::wait_for_task_completion(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &task,
        )
        .await?;

        Ok(())
    }

    /// Gets the instance status.
    async fn status(&self, id: &str) -> Result<InstanceStatus, Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let result = crate::proxmox_api::vm_status_read(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?;
        Ok(result.data.status.into())
    }

    /// Stops the instance.
    async fn stop(&self, id: &str) -> Result<(), Problem> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let task = crate::proxmox_api::vm_status_stop(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Problem::MalformedVmId(id.to_owned()))?,
        )
        .await?
        .data;

        proxmox_api::helpers::wait_for_task_completion(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &task,
        )
        .await?;

        Ok(())
    }
}
