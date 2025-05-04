pub use crate::proxmox_api::cluster_next_id::mock::WithClusterNextId;
pub use crate::proxmox_api::cluster_resources_list::mock::WithClusterResourceList;
pub use crate::proxmox_api::task_status_read::mock::WithTaskStatusReadMock;
pub use crate::proxmox_api::vm_clone::mock::WithVMCloneMock;
pub use crate::proxmox_api::vm_create::mock::WithVMCreateMock;
pub use crate::proxmox_api::vm_delete::mock::WithVMDeleteMock;
pub use crate::proxmox_api::vm_list::mock::WithVMListMock;
pub use crate::proxmox_api::vm_status_read::mock::WithVMStatusReadMock;
pub use crate::proxmox_api::vm_status_start::mock::WithVMStatusStartMock;
pub use crate::proxmox_api::vm_status_stop::mock::WithVMStatusStopMock;

pub struct MockServer {
    pub mocks: Vec<mockito::Mock>,
    pub server: mockito::ServerGuard,
}

impl MockServer {
    pub async fn new() -> Self {
        MockServer {
            server: mockito::Server::new_async().await,
            mocks: vec![],
        }
    }

    /// The URL of the mock server (including the protocol).
    pub fn url(&self) -> String {
        self.server.url()
    }
}
