use crate::Error;
use crate::instance::{Instance, InstanceCreateRequest, Instances, Status};
use crate::proxmox::api;
use crate::proxmox::api::{
    cluster_resources_list::ResourceType, helpers, vm_clone::VMCloneOptions, vm_create::VMConfig,
};
use std::net::Ipv4Addr;

#[derive(Clone)]
pub struct ProxmoxInstanceService {
    pub api_url: String,
    pub client: reqwest::Client,
    pub authorization: String,
}

impl Instances for ProxmoxInstanceService {
    async fn list(&self) -> Result<Vec<Instance>, Error> {
        let response =
            api::cluster_resources_list(&self.api_url, &self.client, &self.authorization, "vm")
                .await?
                .data;
        response
            .into_iter()
            .filter(|resource| resource.resource_type == ResourceType::Qemu)
            .map(|resource| resource.try_into().map_err(Into::into))
            .collect()
    }

    /// Clones the instance.
    async fn clone(&self, id: &str) -> Result<String, Error> {
        let id = id
            .parse::<u32>()
            .map_err(|_| Error::MalformedVmId(id.to_owned()))?;

        let next_id = api::cluster_next_id(&self.api_url, &self.client, &self.authorization)
            .await?
            .data;

        let node_id =
            helpers::get_vm_execution_node(&self.api_url, &self.client, &self.authorization, id)
                .await?;

        let options = VMCloneOptions {
            newid: next_id,
            full: true,
        };

        let task = api::vm_clone(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id,
            &options,
        )
        .await?
        .data;

        api::helpers::wait_for_task_completion(
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
    async fn create(&self, options: InstanceCreateRequest) -> Result<String, Error> {
        let next_id = api::cluster_next_id(&self.api_url, &self.client, &self.authorization)
            .await?
            .data;

        let node_id =
            api::cluster_resources_list(&self.api_url, &self.client, &self.authorization, "node")
                .await?
                .data
                .first()
                .ok_or_else(|| api::Error::NoNodesAvailable)?
                .node
                .clone()
                .expect("node should be defined for resource of type node");

        let vm_config = VMConfig::from_instance_config(options, next_id);

        let task_id = api::vm_create(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            &vm_config,
        )
        .await?
        .data;

        api::helpers::wait_for_task_completion(
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
    async fn get_ip_address(&self, id: &str) -> Result<Option<Ipv4Addr>, Error> {
        // Parse the id to a u32
        let id = id
            .parse::<u32>()
            .map_err(|_| Error::MalformedVmId(id.to_owned()))?;

        // Get the VM execution node
        let node_id =
            helpers::get_vm_execution_node(&self.api_url, &self.client, &self.authorization, id)
                .await?;

        // Get the VM network interfaces
        let ip = api::vm_config_read(
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
    async fn delete(&self, id: &str) -> Result<(), Error> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let task = api::vm_delete(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?
        .data;

        api::helpers::wait_for_task_completion(
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
    async fn start(&self, id: &str) -> Result<(), Error> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let task = api::vm_status_start(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?
        .data;

        api::helpers::wait_for_task_completion(
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
    async fn status(&self, id: &str) -> Result<Status, Error> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let result = api::vm_status_read(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?;
        Ok(result.data.status.into())
    }

    /// Stops the instance.
    async fn stop(&self, id: &str) -> Result<(), Error> {
        let node_id = helpers::get_vm_execution_node(
            &self.api_url,
            &self.client,
            &self.authorization,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?;

        let task = api::vm_status_stop(
            &self.api_url,
            &self.client,
            &self.authorization,
            &node_id,
            id.parse::<u32>()
                .map_err(|_| Error::MalformedVmId(id.to_owned()))?,
        )
        .await?
        .data;

        api::helpers::wait_for_task_completion(
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
