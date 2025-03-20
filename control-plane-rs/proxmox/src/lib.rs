mod api_response;
mod cluster;
mod endpoints;
mod error;
mod node;
mod vm;

pub use cluster::Cluster;
pub use node::Node;
pub use vm::VM;

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::endpoints::vm_create::mock::WithVMCreateMock;
    pub use crate::endpoints::vm_delete::mock::WithVMDeleteMock;
    pub use crate::endpoints::vm_list::mock::WithVMListMock;
    pub use crate::endpoints::vm_status_read::mock::WithVMStatusReadMock;
    pub use crate::endpoints::vm_status_start::mock::WithVMStatusStartMock;
    pub use crate::endpoints::vm_status_stop::mock::WithVMStatusStopMock;

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
}
