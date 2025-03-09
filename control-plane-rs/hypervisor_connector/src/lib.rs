pub mod error;
pub mod hypervisor;
pub mod proxmox;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

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
