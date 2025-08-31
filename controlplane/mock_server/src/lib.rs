//! Shared mock server utilities for testing across the controlplane workspace.
//!
//! This module provides a centralized mock server implementation used by all
//! crates in the controlplane workspace for testing HTTP endpoints, OIDC
//! authentication, and API integrations.
//!
//! ## Key Features
//!
//! - Lightweight wrapper around `mockito::Server`
//! - Consistent URL generation for test scenarios
//! - Shared mock collection management
//!
//! ## Design Philosophy
//!
//! The mock server was extracted from individual crates to provide a unified
//! testing approach across the workspace, ensuring consistent behavior and
//! reducing duplication.
//!
//! ## Usage Pattern
//!
//! ```
//! use mock_server::MockServer;
//!
//! # async fn test_with_mock() {
//! let mock = MockServer::new().await;
//! // Use mock.url() to get the base URL for your tests
//! // Configure specific endpoints using mockito traits
//! # }
//! ```

pub struct MockServer {
    pub mocks: Vec<mockito::Mock>,
    pub server: mockito::ServerGuard,
}

impl MockServer {
    /// Creates a new mock server instance.
    ///
    /// Initializes a fresh mockito server with an empty mock collection,
    /// ready for test configuration.
    pub async fn new() -> Self {
        MockServer {
            server: mockito::Server::new_async().await,
            mocks: vec![],
        }
    }

    /// The URL of the mock server (including the protocol).
    ///
    /// Returns the base URL that can be used to configure clients
    /// to connect to this mock server instance.
    pub fn url(&self) -> String {
        self.server.url()
    }
}
