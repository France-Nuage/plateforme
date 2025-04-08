# IAM Integration Guide

This guide explains how to integrate the IAM package into existing gRPC services in your project.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Basic Integration](#basic-integration)
3. [Configuration Options](#configuration-options)
4. [Using Multiple Providers](#using-multiple-providers)
5. [Error Handling](#error-handling)
6. [Testing Services with Authentication](#testing-services-with-authentication)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

Before integrating IAM, make sure you have:

- Created OAuth client credentials for your provider (e.g., Google OAuth Console, GitHub OAuth Apps)
- Added the IAM crate to your dependencies
- A basic understanding of Tonic gRPC services 

## Basic Integration

### 1. Add the IAM dependency to your service

```toml
[dependencies]
iam = { path = "../iam" }
```

### 2. Set up environment variables

For Google OAuth2:

```
OAUTH_CLIENT_ID=your-google-client-id
OAUTH_CLIENT_SECRET=your-google-client-secret
OAUTH_REDIRECT_URI=https://your-app.com/oauth/callback
```

### 3. Configure the auth provider and interceptor

```rust
use iam::{
    auth::AuthConfig,
    interceptor::AuthInterceptor,
    providers::google::GoogleAuthProvider,
};

// Load config from environment variables
let auth_config = AuthConfig::from_env()?;
let auth_provider = GoogleAuthProvider::new(auth_config);

// Create an interceptor with the auth provider
let auth_interceptor = AuthInterceptor::new(auth_provider);
```

### 4. Apply the interceptor to your service

```rust
let your_service = YourServiceServer::new(YourServiceImpl::default())
    .with_interceptor(auth_interceptor);

Server::builder()
    .add_service(your_service)
    .serve("0.0.0.0:50051".parse()?)
    .await?;
```

### 5. Access user information in service methods

```rust
use iam::context::RequestContextExt;

#[tonic::async_trait]
impl YourService for YourServiceImpl {
    async fn your_method(&self, request: Request<YourRequest>) -> Result<Response<YourResponse>, Status> {
        // Get the request context
        let context = request.context();
        
        // Check if the user is authenticated
        if !context.is_authenticated() {
            return Err(Status::unauthenticated("User not authenticated"));
        }
        
        // Get the user information
        let user_info = context.user_info().unwrap();
        
        // Use user information in your logic
        println!("Request from user: {}", user_info.email);
        
        // ... your service logic ...
        
        Ok(Response::new(YourResponse { /* ... */ }))
    }
}
```

## Configuration Options

### Programmatic Configuration

Instead of loading from environment variables, you can create the configuration programmatically:

```rust
let auth_config = AuthConfig::new(
    "your-client-id",
    "your-client-secret",
    "https://your-app.com/oauth/callback",
)
.with_param("additional_param", "value");

let auth_provider = GoogleAuthProvider::new(auth_config);
```

### Optional Authentication

You can make authentication optional, allowing requests without a valid token to proceed without user info:

```rust
let auth_interceptor = AuthInterceptor::optional(auth_provider);
```

This is useful for APIs that have both public and protected endpoints.

## Using Multiple Providers

You can support multiple OAuth2 providers using a custom router provider:

```rust
use iam::auth::{AuthProvider, AuthConfig, TokenInfo};
use iam::error::{AuthError, AuthResult};
use iam::user::UserInfo;
use async_trait::async_trait;

struct MultiProviderAuth {
    google_provider: GoogleAuthProvider,
    github_provider: GitHubAuthProvider,
}

impl MultiProviderAuth {
    fn new(google_config: AuthConfig, github_config: AuthConfig) -> Self {
        Self {
            google_provider: GoogleAuthProvider::new(google_config),
            github_provider: GitHubAuthProvider::new(github_config),
        }
    }
    
    // Helper method to determine provider from token format or header
    fn select_provider(&self, token: &str, provider_hint: Option<&str>) -> &dyn AuthProvider {
        // Use provider_hint if available
        if let Some(hint) = provider_hint {
            match hint {
                "google" => &self.google_provider,
                "github" => &self.github_provider,
                _ => &self.google_provider, // Default to Google
            }
        } else {
            // Try to guess based on token format
            if token.split('.').count() == 3 {
                // Likely a JWT (Google)
                &self.google_provider
            } else {
                // Try GitHub by default
                &self.github_provider
            }
        }
    }
}

#[async_trait]
impl AuthProvider for MultiProviderAuth {
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo> {
        // Try each provider in sequence
        match self.google_provider.validate_token(token).await {
            Ok(user_info) => Ok(user_info),
            Err(_) => self.github_provider.validate_token(token).await,
        }
    }
    
    // Implement other methods...
}
```

## Error Handling

IAM errors are automatically converted to appropriate tonic `Status` values:

- `AuthError::InvalidToken` -> `Status::unauthenticated`
- `AuthError::TokenExpired` -> `Status::unauthenticated`
- `AuthError::AccessDenied` -> `Status::permission_denied`
- `AuthError::ConfigError` -> `Status::failed_precondition`
- Other errors -> `Status::internal`

You can also convert errors manually when needed:

```rust
fn handle_auth_error(err: AuthError) -> Status {
    Status::from(err)
}
```

## Testing Services with Authentication

### Mock Provider

Use the `mockall` crate to create a mock provider for testing:

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    AuthProvider {}

    #[async_trait]
    impl AuthProvider for AuthProvider {
        async fn validate_token(&self, token: &str) -> AuthResult<UserInfo>;
        async fn refresh_token(&self, refresh_token: &str) -> AuthResult<TokenInfo>;
        fn authorize_url(&self, state: &str, scopes: &[&str]) -> String;
        async fn exchange_code(&self, code: &str) -> AuthResult<TokenInfo>;
        async fn get_user_info(&self, token: &str) -> AuthResult<UserInfo>;
    }
}

#[tokio::test]
async fn test_service_with_auth() {
    // Create a mock provider
    let mut mock_provider = MockAuthProvider::new();
    
    // Set up expectations
    mock_provider
        .expect_validate_token()
        .with(eq("test_token"))
        .returning(|_| Ok(UserInfo::new("test_user", "test@example.com", true)));
    
    // Create an interceptor with the mock provider
    let auth_interceptor = AuthInterceptor::new(mock_provider);
    
    // Create your service with the interceptor
    let service = YourServiceServer::with_interceptor(
        YourServiceImpl::default(),
        auth_interceptor,
    );
    
    // Test your service with a valid token
    let request = Request::new(YourRequest { /* ... */ });
    request.metadata_mut().insert(
        "authorization",
        "Bearer test_token".parse().unwrap(),
    );
    
    // Perform the request and assert the response
}
```

## Troubleshooting

### Common Issues

1. **Authentication Fails**
   
   Check that:
   - The token is valid and not expired
   - The Authorization header is formatted correctly: `Bearer your-token`
   - The client ID and secret match the provider's records

2. **Cannot Access User Info**

   Check that:
   - The request has passed through the auth interceptor
   - The token has the necessary scopes (e.g., `profile`, `email`)
   - You're using `request.context()` to get the context

3. **Service Returns Unauthenticated Status**

   Possible causes:
   - No token provided in the request
   - Token is invalid or expired
   - AuthInterceptor is configured to require authentication

4. **Performance Issues**

   - Token validation is performed asynchronously, so it shouldn't block
   - Consider caching validation results for frequently used tokens
   - Use connection pooling in the HTTP client for provider requests