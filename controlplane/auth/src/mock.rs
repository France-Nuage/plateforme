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
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use mock_server::MockServer;
use rsa::PublicKeyParts;
use serde_json::json;

use crate::OpenID;

/// Standard key identifier used for mock JWT signatures.
///
/// This constant provides a consistent key ID for RSA keys generated during testing.
/// All mock JWTs created by the `JwkValidator::token()` method will use this key ID,
/// and the corresponding JWK exposed by `WithJwks::with_jwks()` will have the same ID.
pub const MOCK_JWK_KID: &str = "mock-key-01";

/// Trait for configuring JWK Set endpoints on mock servers.
///
/// This trait extends mock servers with the ability to respond to JWK Set requests
/// at the standard OAuth discovery endpoint (`/oauth/discovery/keys`). The exposed
/// JWK Set contains the public key corresponding to the private key used for
/// signing mock JWT tokens.
///
/// ## Key Management
///
/// The JWK Set uses the same RSA key pair as `JwkValidator::rsa()`, ensuring that
/// tokens created with `JwkValidator::token()` can be validated against the
/// JWK Set exposed by this mock endpoint.
pub trait WithJwks {
    /// Configures the mock server to respond to JWK Set requests.
    ///
    /// Adds a mock endpoint at `/oauth/discovery/keys` that returns a JWK Set
    /// containing the public RSA key used for JWT signature validation in tests.
    ///
    /// # Returns
    ///
    /// The configured mock server with JWK Set endpoint enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # mod wrapper_module {
    /// use auth::mock::WithJwks;
    /// use mock_server::MockServer;
    ///
    /// #[tokio::test]
    /// async fn test_jwt_validation() {
    ///     let mock = MockServer::new().await.with_jwks();
    ///     // Mock server now responds to /oauth/discovery/keys with JWK Set
    /// }
    /// # }
    /// ```
    fn with_jwks(self) -> Self;
}

impl WithJwks for MockServer {
    fn with_jwks(mut self) -> Self {
        let (_, public_key) = OpenID::rsa();
        let key = json!({
            "kty": "RSA",
            "use": "sig",
            "kid": MOCK_JWK_KID,
            "alg": "RS256",
            "n": URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be()),
            "e": URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be()),
        });
        let body = json!({ "keys": [key] }).to_string();
        let mock = self
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/oauth/discovery/keys$".to_string()),
            )
            .with_body(body)
            .create();
        self.mocks.push(mock);
        self
    }
}

/// Trait for configuring OIDC well-known discovery endpoint on mock servers.
///
/// This trait extends mock servers with the ability to respond to OIDC
/// discovery requests, enabling authentication testing workflows.
pub trait WithWellKnown {
    /// Configures the mock server to respond to OIDC discovery requests.
    ///
    /// Adds a mock endpoint that responds to `/.well-known/openid-configuration`
    /// with a GitLab-compatible OIDC provider configuration.
    fn with_well_known(self) -> Self;
}

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
