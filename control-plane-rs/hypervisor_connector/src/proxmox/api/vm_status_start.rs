use crate::error::Error;

pub async fn vm_status_start(
    api_url: &str,
    client: &reqwest::Client,
    node_id: &str,
    vm_id: u32,
) -> Result<(), Error> {
    client
        .post(format!(
            "{}/api2/json/nodes/{}/qemu/{}/status/start",
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

    trait WithVMStatusStartMock {
        fn with_vm_status_start(self) -> Self;
    }

    impl WithVMStatusStartMock for MockServer {
        fn with_vm_status_start(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.*/qemu/\d+/status/start$".to_string(),
                    ),
                )
                .with_body(r#"{"data":"UPID:pve-node1:0021B91A:023305D3:67CC7C84:qmstart:105:root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_status_start();
        let result = vm_status_start(&server.url(), &client, "pve-node1", 100).await;

        assert!(result.is_ok());
    }
}
