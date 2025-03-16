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
    fn instance(&self, id: u32) -> impl hypervisor::Instance {
        crate::vm::VM::new(self.api_url, self.client, id, self)
    }

    async fn list_instances(&self) -> Result<(), hypervisor::error::Error> {
        crate::endpoints::vm_list(self.api_url, self.client, self.id)
            .await
            .unwrap();

        Ok(())
    }
}
