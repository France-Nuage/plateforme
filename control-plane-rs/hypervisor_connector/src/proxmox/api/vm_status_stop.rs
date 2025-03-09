use crate::error::Error;

pub async fn vm_status_stop(
    api_url: &str,
    client: &reqwest::Client,
    node_id: &str,
    vm_id: u32,
) -> Result<(), Error> {
    client
        .post(format!(
            "{}/api2/json/nodes/{}/qemu/{}/status/stop",
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

    trait WithVMStatusStopMock {
        fn with_vm_status_stop(self) -> Self;
    }

    impl WithVMStatusStopMock for MockServer {
        fn with_vm_status_stop(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.*/qemu/\d+/status/stop$".to_string(),
                    ),
                )
                .with_body(r#"{"data":"UPID:pve-node1:0021BBE8:02333375:67CC7CF9:qmstop:105:root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_status_stop();
        let result = vm_status_stop(&server.url(), &client, "pve-node1", 100).await;

        assert!(result.is_ok());
    }
}
