//! Example showing how to create a custom OAuth2 provider and use it with the IAM package.
//!
//! This example demonstrates how to implement a custom OAuth2 provider (GitHub in this case)
//! and use it with the IAM package. In a real implementation, you would generate proper gRPC
//! code from protobuf definitions.

use async_trait::async_trait;
use iam::{
    auth::{AuthConfig, AuthProvider, TokenInfo},
    context::RequestContextExt,
    error::{AuthError, AuthResult},
    interceptor::AuthInterceptor,
    user::UserInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tonic::{service::Interceptor, Request, Response, Status};

/// GitHub user info response structure.
#[derive(Debug, Serialize, Deserialize)]
struct GitHubUserInfo {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    #[serde(flatten)]
    additional_fields: HashMap<String, serde_json::Value>,
}

/// Custom GitHub OAuth2 provider implementation.
#[derive(Debug, Clone)]
struct GitHubAuthProvider {
    config: AuthConfig,
    client: reqwest::Client,
}

impl GitHubAuthProvider {
    /// Creates a new GitHub auth provider.
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Gets user information from the GitHub API.
    async fn get_github_user(&self, token: &str) -> AuthResult<GitHubUserInfo> {
        let response = self
            .client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "IAM-Example")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(AuthError::HttpError)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::ProviderError(format!(
                "GitHub API error: {} - {}",
                status, error_text
            )));
        }

        response.json::<GitHubUserInfo>().await.map_err(AuthError::HttpError)
    }

    /// Converts GitHub user info to the common UserInfo format.
    fn convert_github_user(github_user: GitHubUserInfo) -> UserInfo {
        let email = github_user.email.unwrap_or_else(|| format!("{}@github.com", github_user.login));
        
        let mut user_info = UserInfo::new(
            github_user.id.to_string(),
            email,
            true, // GitHub emails are verified
        );

        if let Some(name) = github_user.name {
            user_info = user_info.with_name(name);
        }

        if let Some(avatar_url) = github_user.avatar_url {
            user_info = user_info.with_picture(avatar_url);
        }

        // Add GitHub username as a claim
        user_info = user_info.with_claim("github_username", serde_json::Value::String(github_user.login));

        user_info
    }
}

#[async_trait]
impl AuthProvider for GitHubAuthProvider {
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo> {
        let github_user = self.get_github_user(token).await?;
        Ok(Self::convert_github_user(github_user))
    }

    async fn refresh_token(&self, _refresh_token: &str) -> AuthResult<TokenInfo> {
        // GitHub doesn't provide refresh tokens in the standard OAuth flow
        Err(AuthError::ProviderError(
            "GitHub doesn't support refresh tokens in standard OAuth".to_string(),
        ))
    }

    fn authorize_url<'a>(&self, state: &str, scopes: &[&'a str]) -> String {
        let scopes_str = scopes.join(" ");
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&scopes_str),
            urlencoding::encode(state)
        )
    }

    async fn exchange_code(&self, code: &str) -> AuthResult<TokenInfo> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
        ];

        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(AuthError::HttpError)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::ProviderError(format!(
                "GitHub OAuth error: {} - {}",
                status, error_text
            )));
        }

        #[derive(Deserialize)]
        struct GitHubTokenResponse {
            access_token: String,
            token_type: String,
            scope: Option<String>,
            error: Option<String>,
            error_description: Option<String>,
        }

        let token_response = response
            .json::<GitHubTokenResponse>()
            .await
            .map_err(AuthError::HttpError)?;

        if let Some(error) = token_response.error {
            let description = token_response.error_description.unwrap_or_else(|| "Unknown error".to_string());
            return Err(AuthError::ProviderError(format!(
                "GitHub OAuth error: {} - {}",
                error, description
            )));
        }

        Ok(TokenInfo {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            refresh_token: None, // GitHub doesn't provide refresh tokens
            expires_in: None,    // GitHub tokens don't expire by default
            scope: token_response.scope,
            id_token: None,      // GitHub doesn't provide ID tokens
        })
    }

    async fn get_user_info(&self, token: &str) -> AuthResult<UserInfo> {
        let github_user = self.get_github_user(token).await?;
        Ok(Self::convert_github_user(github_user))
    }
}

/// Example of using the GitHub auth provider
fn example_usage() -> Result<(), Box<dyn Error>> {
    // Configure the GitHub auth provider
    let auth_config = AuthConfig::new(
        "your-github-client-id",  // Replace with your client ID
        "your-github-client-secret", // Replace with your client secret
        "http://localhost:8080/oauth/callback", // Replace with your redirect URI
    );
    
    // Create the GitHub auth provider
    let auth_provider = GitHubAuthProvider::new(auth_config);
    
    // Create an auth interceptor that requires authentication
    let required_auth = AuthInterceptor::new(auth_provider.clone());
    
    // Or, create an auth interceptor that makes authentication optional
    let optional_auth = AuthInterceptor::optional(auth_provider);
    
    println!("Auth interceptors created successfully");
    
    // In a real implementation, you would use the interceptor with tonic
    
    Ok(())
}

/// Example of how to access GitHub-specific user information in a gRPC service method
fn example_service_method(request: Request<()>) -> Result<Response<()>, Status> {
    // Get the request context
    let context = request.context();
    
    // Check if the user is authenticated
    if context.is_authenticated() {
        // Get the user information
        let user = context.user_info().unwrap();
        
        println!("Request from user: {}", user.email);
        println!("User ID: {}", user.id);
        
        // Get the GitHub username from the custom claim
        if let Some(github_username) = user.get_claim::<String>("github_username") {
            println!("GitHub username: {}", github_username);
        }
        
        // Use the user information to implement your business logic
        // ...
    } else {
        println!("Request from unauthenticated user");
        // Handle unauthenticated requests
        // ...
    }
    
    // Process the request and return a response
    Ok(Response::new(()))
}

fn main() {
    println!("This is a code example and not meant to be run directly.");
    println!("See the code for examples of how to use the IAM package with a custom OAuth2 provider.");
}