pub struct Cluster<'a> {
    api_url: &'a str,
    client: &'a reqwest::Client,
}

impl<'a> Cluster<'a> {
    /// Creates a new Proxmox cluster.
    pub fn new(api_url: &'a str, client: &'a reqwest::Client) -> Self {
        Cluster { api_url, client }
    }
}

impl hypervisor::Cluster for Cluster<'_> {
    fn node(&self, id: &str) -> impl hypervisor::Node {
        crate::node::Node::new(self.api_url, self.client, id)
    }
}
