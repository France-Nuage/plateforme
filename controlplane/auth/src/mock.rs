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
//! use auth::mock::WithWellKnown;
//! use mock_server::MockServer;
//!
//! #[tokio::test]
//! async fn test_auth() {
//!     let mock = MockServer::new().await.with_well_known();
//!     // Mock server now responds to /.well-known/openid-configuration
//! }
//! ```

use mock_server::MockServer;

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
        let mock = self
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/.well-known/openid-configuration$".to_string()),
            )
            .with_body(r#"{"issuer":"https://gitlab.com","authorization_endpoint":"https://gitlab.com/oauth/authorize","token_endpoint":"https://gitlab.com/oauth/token","revocation_endpoint":"https://gitlab.com/oauth/revoke","introspection_endpoint":"https://gitlab.com/oauth/introspect","userinfo_endpoint":"https://gitlab.com/oauth/userinfo","jwks_uri":"https://gitlab.com/oauth/discovery/keys","scopes_supported":["api","read_api","read_user","create_runner","manage_runner","k8s_proxy","self_rotate","mcp","read_repository","write_repository","read_registry","write_registry","read_virtual_registry","write_virtual_registry","read_observability","write_observability","ai_features","sudo","admin_mode","read_service_ping","openid","profile","email","ai_workflows","user:*"],"response_types_supported":["code"],"response_modes_supported":["query","fragment","form_post"],"grant_types_supported":["authorization_code","password","client_credentials","device_code","refresh_token"],"token_endpoint_auth_methods_supported":["client_secret_basic","client_secret_post"],"subject_types_supported":["public"],"id_token_signing_alg_values_supported":["RS256"],"claim_types_supported":["normal"],"claims_supported":["iss","sub","aud","exp","iat","sub_legacy","name","nickname","preferred_username","email","email_verified","website","profile","picture","groups","groups_direct","https://gitlab.org/claims/groups/owner","https://gitlab.org/claims/groups/maintainer","https://gitlab.org/claims/groups/developer"],"code_challenge_methods_supported":["plain","S256"]}"#)
            .create();
        self.mocks.push(mock);
        self
    }
}
