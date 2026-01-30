use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::Serialize;

/// Request body for the Proxmox disk resize endpoint.
#[derive(Debug, Serialize)]
struct VMDiskResizeRequest<'a> {
    /// The disk to resize (e.g. `scsi0`, `virtio0`).
    disk: &'a str,
    /// The new size in Proxmox format (e.g. `50G`).
    size: &'a str,
}

/// Resizes a VM disk.
///
/// This is required after creating a VM with `import-from`, as Proxmox ignores
/// the `size` parameter during import and uses the source image size instead.
///
/// Calls `PUT /nodes/{node}/qemu/{vmid}/resize`.
pub async fn vm_disk_resize(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vm_id: u32,
    disk: &str,
    size_bytes: u64,
) -> Result<ApiResponse<String>, Error> {
    let size_gb = size_bytes / (1024 * 1024 * 1024);
    let size = &format!("{}G", size_gb);
    let body = VMDiskResizeRequest { disk, size };

    client
        .put(format!(
            "{}/api2/json/nodes/{}/qemu/{}/resize",
            api_url, node_id, vm_id
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(&body)
        .send()
        .await
        .to_api_response()
        .await
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithVMDiskResizeMock {
        fn with_vm_disk_resize(self) -> Self;
    }

    impl WithVMDiskResizeMock for MockServer {
        fn with_vm_disk_resize(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "PUT",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/qemu/\d+/resize$".to_string()),
                )
                .with_body(r#"{"data":""}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithVMDiskResizeMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_vm_disk_resize() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_disk_resize();
        let result = vm_disk_resize(
            &server.url(),
            &client,
            "",
            "pve-node1",
            100,
            "scsi0",
            53687091200,
        )
        .await;

        assert!(result.is_ok());
    }
}
