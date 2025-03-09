use crate::hypervisor::{Hypervisor, Node};

use super::ProxmoxNode;

pub struct ProxmoxCluster<'a> {
    api_url: &'a str,
    client: &'a reqwest::Client,
}

impl<'a> ProxmoxCluster<'a> {
    /// Creates a new Proxmox cluster.
    pub fn new(api_url: &'a str, client: &'a reqwest::Client) -> Self {
        ProxmoxCluster { api_url, client }
    }
}

impl Hypervisor for ProxmoxCluster<'_> {
    fn node(&self, id: &str) -> impl Node {
        ProxmoxNode::new(self.api_url, self.client, id)
    }
}
