use super::ProxmoxVM;
use crate::error::Error;
use crate::hypervisor::{Instance, Node};
use crate::proxmox::api;

pub struct ProxmoxNode<'a, 'b> {
    api_url: &'a str,
    client: &'a reqwest::Client,
    pub id: &'b str,
}

impl<'a, 'b> ProxmoxNode<'a, 'b> {
    pub fn new(api_url: &'a str, client: &'a reqwest::Client, id: &'b str) -> Self {
        ProxmoxNode {
            api_url,
            client,
            id,
        }
    }
}

impl Node for ProxmoxNode<'_, '_> {
    fn instance(&self, id: u32) -> impl Instance {
        ProxmoxVM::new(self.api_url, self.client, id, self)
    }

    async fn list_instances(&self) -> Result<(), Error> {
        api::vm_list(self.api_url, self.client, self.id).await
    }
}
