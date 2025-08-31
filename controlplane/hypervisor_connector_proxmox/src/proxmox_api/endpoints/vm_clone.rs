use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};
use serde::Serialize;

pub async fn vm_clone(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vmid: u32,
    options: &VMCloneOptions,
) -> Result<ApiResponse<String>, crate::proxmox_api::problem::Problem> {
    client
        .post(format!(
            "{}/api2/json/nodes/{}/qemu/{}/clone",
            api_url, node_id, vmid
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(options)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Serialize)]
pub struct VMCloneOptions {
    pub newid: u32,
    pub full: bool,
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithVMCloneMock {
        fn with_vm_clone(self) -> Self;
    }

    impl WithVMCloneMock for MockServer {
        fn with_vm_clone(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/qemu/.*/clone$".to_string()),
                )
                .with_body(r#"{"data":"UPID:pve-node1:0021B19E:02328820:67CC7B42:qmclone:100:root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithVMCloneMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_clone();
        let options = VMCloneOptions {
            newid: 101,
            full: true,
        };
        let result = vm_clone(&server.url(), &client, "", "pve-node1", 100, &options).await;

        assert!(result.is_ok());
    }
}
