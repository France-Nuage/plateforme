use crate::error::Error;
use crate::hypervisor::{Instance, InstanceConfig, InstanceStatus};
use crate::proxmox::ProxmoxNode;
use crate::proxmox::api;

pub struct ProxmoxVM<'a, 'b> {
    api_url: &'a str,
    client: &'a reqwest::Client,
    id: u32,
    node: &'a ProxmoxNode<'a, 'b>,
}

impl<'a, 'b> ProxmoxVM<'a, 'b> {
    pub fn new(
        api_url: &'a str,
        client: &'a reqwest::Client,
        id: u32,
        node: &'a ProxmoxNode<'a, 'b>,
    ) -> Self {
        ProxmoxVM {
            api_url,
            client,
            id,
            node,
        }
    }
}

impl Instance for ProxmoxVM<'_, '_> {
    /// Creates the instance.
    async fn create(&self, options: &InstanceConfig<'_>) -> Result<(), Error> {
        api::vm_create(
            self.api_url,
            self.client,
            self.node.id,
            &api::VMConfig::from(options),
        )
        .await
    }

    /// Deletes the instance.
    async fn delete(&self) -> Result<(), Error> {
        api::vm_delete(self.api_url, self.client, self.node.id, self.id).await
    }

    /// Starts the instance.
    async fn start(&self) -> Result<(), Error> {
        api::vm_status_start(self.api_url, self.client, self.node.id, self.id).await
    }

    /// Gets the instance status.
    async fn status(&self) -> Result<InstanceStatus, Error> {
        api::vm_status_read(self.api_url, self.client, self.node.id, self.id).await
    }

    /// Stops the instance.
    async fn stop(&self) -> Result<(), Error> {
        api::vm_status_stop(self.api_url, self.client, self.node.id, self.id).await
    }
}
