mod api_response;
mod cluster;
mod endpoints;
mod error;
mod node;
mod vm;

pub use cluster::Cluster;
pub use node::Node;
pub use vm::VM;

#[cfg(test)]
pub mod tests {
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
