use crate::error::Error;

pub async fn vm_delete(
    api_url: &str,
    client: &reqwest::Client,
    node_id: &str,
    vm_id: u32,
) -> Result<(), Error> {
    client
        .delete(format!(
            "{}/api2/json/nodes/{}/qemu/{}",
            api_url, node_id, vm_id
        ))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::MockServer;

    trait WithVMDeleteMock {
        fn with_vm_delete(self) -> Self;
    }

    impl WithVMDeleteMock for MockServer {
        fn with_vm_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/qemu/\d+$".to_string()),
                )
                .with_body(r#"{"data":"UPID:pve-node1:0021B19A:02328725:67CC7B3F:qmdestroy:110:root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_delete();
        let result = vm_delete(&server.url(), &client, "pve-node1", 100).await;

        assert!(result.is_ok());
    }
}
