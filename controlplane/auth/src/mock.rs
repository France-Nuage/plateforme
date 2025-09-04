//! Mock OIDC provider functionality for testing authentication workflows.
//!
//! This module provides mock implementations of OIDC provider endpoints,
//! specifically the well-known configuration endpoint that's essential for
//! JWT validation during testing.
//!
//! ## Key Features
//!
//! - **Well-Known Configuration**: Mock OIDC discovery endpoint
//! - **GitLab-Compatible**: Uses GitLab OIDC response format
//! - **Integration Ready**: Works with the shared mock_server infrastructure
//!
//! ## Usage Pattern
//!
//! ```
//! # #[cfg(feature = "mock")]
//! # mod wrapper_module {
//! use auth::mock::WithWellKnown;
//! use mock_server::MockServer;
//!
//! #[tokio::test]
//! async fn test_auth() {
//!     let mock = MockServer::new().await.with_well_known();
//!     // Mock server now responds to /.well-known/openid-configuration
//! }
//! # }
//! ```

#[cfg(feature = "mock")]
use mock_server::MockServer;

#[cfg(feature = "mock")]
use serde_json::json;

/// Trait for configuring OIDC well-known discovery endpoint on mock servers.
///
/// This trait extends mock servers with the ability to respond to OIDC
/// discovery requests, enabling authentication testing workflows.
#[cfg(feature = "mock")]
pub trait WithWellKnown {
    /// Configures the mock server to respond to OIDC discovery requests.
    ///
    /// Adds a mock endpoint that responds to `/.well-known/openid-configuration`
    /// with a GitLab-compatible OIDC provider configuration.
    fn with_well_known(self) -> Self;
}

#[cfg(feature = "mock")]
impl WithWellKnown for MockServer {
    fn with_well_known(mut self) -> Self {
        let base = self.url();
        let body = json!({
            "issuer": base,
            "authorization_endpoint": format!("{base}/oauth/authorize"),
            "token_endpoint": format!("{base}/oauth/token"),
            "revocation_endpoint": format!("{base}/oauth/revoke"),
            "introspection_endpoint": format!("{base}/oauth/introspect"),
            "userinfo_endpoint": format!("{base}/oauth/userinfo"),
            "jwks_uri": format!("{base}/oauth/discovery/keys"),
            "scopes_supported": ["openid","profile","email"],
            "response_types_supported": ["code"],
            "response_modes_supported": ["query","fragment","form_post"],
            "grant_types_supported": ["authorization_code","client_credentials","refresh_token"],
            "token_endpoint_auth_methods_supported": ["client_secret_basic","client_secret_post"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "claim_types_supported": ["normal"],
            "claims_supported": ["iss","sub","aud","exp","iat","name","preferred_username","email","email_verified"],
            "code_challenge_methods_supported": ["plain","S256"]
        }).to_string();
        let mock = self
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/.well-known/openid-configuration$".to_string()),
            )
            .with_body(body)
            .create();
        self.mocks.push(mock);
        self
    }
}
