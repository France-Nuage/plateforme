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
    async fn instances(
        &self,
    ) -> Result<Vec<proto::v0::InstanceInfo>, hypervisor::problem::Problem> {
        Ok(
            crate::endpoints::cluster_resource_list(self.api_url, self.client)
                .await?
                .data
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    fn node(&self, id: &str) -> impl hypervisor::Node {
        crate::node::Node::new(self.api_url, self.client, id)
    }
}
