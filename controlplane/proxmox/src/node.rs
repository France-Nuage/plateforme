pub struct Node<'a, 'b> {
    api_url: &'a str,
    client: &'a reqwest::Client,
    pub id: &'b str,
}

impl<'a, 'b> Node<'a, 'b> {
    pub fn new(api_url: &'a str, client: &'a reqwest::Client, id: &'b str) -> Self {
        Node {
            api_url,
            client,
            id,
        }
    }
}

impl hypervisor::Node for Node<'_, '_> {
    fn instance(&self, id: &str) -> impl hypervisor::Instance {
        crate::vm::VM::new(self.api_url, self.client, id.parse().unwrap(), self)
    }

    async fn list_instances(&self) -> Result<(), hypervisor::problem::Problem> {
        crate::endpoints::vm_list(self.api_url, self.client, self.id)
            .await
            .unwrap();

        Ok(())
    }
}
