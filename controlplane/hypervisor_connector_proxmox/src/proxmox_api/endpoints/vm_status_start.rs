use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};

pub async fn vm_status_start(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vm_id: u32,
) -> Result<ApiResponse<String>, crate::proxmox_api::Problem> {
    client
        .post(format!(
            "{}/api2/json/nodes/{}/qemu/{}/status/start",
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
    use mock_server::MockServer;

    pub trait WithVMStatusStartMock {
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
}

#[cfg(test)]
mod tests {
    use super::mock::WithVMStatusStartMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_status_start();
        let result = vm_status_start(&server.url(), &client, "", "pve-node1", 100).await;

        assert!(result.is_ok());
    }
}
