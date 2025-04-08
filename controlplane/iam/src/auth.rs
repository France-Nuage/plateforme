//! Core authentication abstractions.
//!
//! This module defines the provider-agnostic interfaces and configuration
//! for OAuth2 authentication.

use crate::error::{AuthError, AuthResult};
use crate::user::UserInfo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Configuration for OAuth2 authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Client ID for the OAuth2 application.
    pub client_id: String,
    
    /// Client secret for the OAuth2 application.
    pub client_secret: String,
    
    /// Redirect URI for the OAuth2 flow.
    pub redirect_uri: String,
    
    /// Additional provider-specific configuration.
    pub extra_params: HashMap<String, String>,
}

impl AuthConfig {
    /// Creates a new AuthConfig with the specified parameters.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            redirect_uri: redirect_uri.into(),
            extra_params: HashMap::new(),
        }
    }

    /// Adds an extra parameter to the configuration.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_params.insert(key.into(), value.into());
        self
    }

    /// Loads the configuration from environment variables.
    ///
    /// The following environment variables are used:
    /// - `OAUTH_CLIENT_ID`: The client ID for the OAuth2 application.
    /// - `OAUTH_CLIENT_SECRET`: The client secret for the OAuth2 application.
    /// - `OAUTH_REDIRECT_URI`: The redirect URI for the OAuth2 flow.
    ///
    /// Additional provider-specific configuration can be loaded by
    /// subclassing this method.
    pub fn from_env() -> AuthResult<Self> {
        // Load required environment variables
        let client_id = env::var("OAUTH_CLIENT_ID")
            .map_err(|_| AuthError::ConfigError("OAUTH_CLIENT_ID not set".to_string()))?;
        
        let client_secret = env::var("OAUTH_CLIENT_SECRET")
            .map_err(|_| AuthError::ConfigError("OAUTH_CLIENT_SECRET not set".to_string()))?;
        
        let redirect_uri = env::var("OAUTH_REDIRECT_URI")
            .map_err(|_| AuthError::ConfigError("OAUTH_REDIRECT_URI not set".to_string()))?;

        Ok(Self::new(client_id, client_secret, redirect_uri))
    }
}

/// Token information returned from an OAuth2 provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// The access token.
    pub access_token: String,
    
    /// The type of token, typically "Bearer".
    pub token_type: String,
    
    /// The refresh token, if provided.
    pub refresh_token: Option<String>,
    
    /// The expiration time in seconds from when the token was issued.
    pub expires_in: Option<u64>,
    
    /// The scopes the token has access to.
    pub scope: Option<String>,
    
    /// The ID token for OpenID Connect providers.
    pub id_token: Option<String>,
}

/// Provider-agnostic interface for OAuth2 authentication.
///
/// This trait defines the operations that any OAuth2 provider must implement,
/// creating a clean abstraction layer that can support multiple providers.
#[async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    /// Validates an access token and returns the user information if valid.
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo>;

    /// Refreshes an access token using a refresh token.
    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<TokenInfo>;

    /// Generates an authorization URL for the OAuth2 flow.
    fn authorize_url<'a>(&self, state: &str, scopes: &[&'a str]) -> String;

    /// Exchanges an authorization code for tokens.
    async fn exchange_code(&self, code: &str) -> AuthResult<TokenInfo>;

    /// Retrieves user information using an access token.
    async fn get_user_info(&self, token: &str) -> AuthResult<UserInfo>;
}