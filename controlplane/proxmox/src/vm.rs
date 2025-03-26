pub struct VM<'a, 'b> {
    api_url: &'a str,
    client: &'a reqwest::Client,
    id: u32,
    node: &'a crate::node::Node<'a, 'b>,
}

impl<'a, 'b> VM<'a, 'b> {
    pub fn new(
        api_url: &'a str,
        client: &'a reqwest::Client,
        id: u32,
        node: &'a crate::node::Node<'a, 'b>,
    ) -> Self {
        VM {
            api_url,
            client,
            id,
            node,
        }
    }
}

impl hypervisor::Instance for VM<'_, '_> {
    /// Creates the instance.
    async fn create(
        &self,
        options: &hypervisor::InstanceConfig<'_>,
    ) -> Result<(), hypervisor::problem::Problem> {
        crate::endpoints::vm_create(
            self.api_url,
            self.client,
            self.node.id,
            &crate::endpoints::VMConfig::from(options),
        )
        .await?;

        Ok(())
    }

    /// Deletes the instance.
    async fn delete(&self) -> Result<(), hypervisor::problem::Problem> {
        crate::endpoints::vm_delete(self.api_url, self.client, self.node.id, self.id).await?;
        Ok(())
    }

    /// Starts the instance.
    async fn start(&self) -> Result<(), hypervisor::problem::Problem> {
        crate::endpoints::vm_status_start(self.api_url, self.client, self.node.id, self.id).await?;
        Ok(())
    }

    /// Gets the instance status.
    async fn status(&self) -> Result<hypervisor::InstanceStatus, hypervisor::problem::Problem> {
        let result =
            crate::endpoints::vm_status_read(self.api_url, self.client, self.node.id, self.id)
                .await?;
        Ok(result.data.status)
    }

    /// Stops the instance.
    async fn stop(&self) -> Result<(), hypervisor::problem::Problem> {
        crate::endpoints::vm_status_stop(self.api_url, self.client, self.node.id, self.id).await?;
        Ok(())
    }
}
