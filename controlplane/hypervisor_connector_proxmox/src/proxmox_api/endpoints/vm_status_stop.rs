use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};

pub async fn vm_status_stop(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vm_id: u32,
) -> Result<ApiResponse<String>, crate::proxmox_api::Problem> {
    client
        .post(format!(
            "{}/api2/json/nodes/{}/qemu/{}/status/stop",
            api_url, node_id, vm_id
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::mock::MockServer;

    pub trait WithVMStatusStopMock {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockServer, WithVMStatusStopMock};

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_status_stop();
        let result = vm_status_stop(&server.url(), &client, "", "pve-node1", 100).await;

        assert!(result.is_ok());
    }
}
