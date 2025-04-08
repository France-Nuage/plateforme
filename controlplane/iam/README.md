# IAM - Identity and Access Management

This crate provides OAuth2 authentication and authorization capabilities for gRPC services. It is designed with a provider-agnostic architecture that supports multiple OAuth2 providers through a clean abstraction layer.

## Features

- **Provider-Agnostic Design**: Core OAuth2 functionality is abstracted through traits, allowing support for multiple providers
- **Google OAuth2 Support**: Built-in implementation for Google OAuth2
- **Extensible Architecture**: Easily add support for additional OAuth2 providers
- **Tonic Integration**: Seamless integration with Tonic gRPC services via interceptors
- **Comprehensive Error Handling**: Clear, typed errors with proper status code mapping
- **Async/Await**: All network operations are non-blocking
- **Normalized User Info**: Consistent user information structure across different providers
- **Token Validation**: Support for both access tokens and ID tokens (JWT)
- **Full OAuth2 Flow**: Support for authorization URL generation, code exchange, token refresh, and validation

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
iam = { path = "../iam" }
```

## Usage

### Basic Setup

```rust
use iam::{
    auth::{AuthConfig, AuthProvider},
    interceptor::AuthInterceptor,
    providers::google::GoogleAuthProvider,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the auth provider
    let auth_config = AuthConfig::from_env()?;
    let auth_provider = GoogleAuthProvider::new(auth_config);

    // Create an interceptor with the auth provider
    let auth_interceptor = AuthInterceptor::new(auth_provider);

    // Use the interceptor with your gRPC service
    let service = YourServiceImpl::default()
        .with_interceptor(auth_interceptor);

    // Start the server
    Server::builder()
        .add_service(service)
        .serve("0.0.0.0:50051".parse()?)
        .await?;

    Ok(())
}
```

### Configuration

The `AuthConfig` struct provides configuration for OAuth2 providers. It can be created programmatically or loaded from environment variables:

```rust
// From environment variables
let config = AuthConfig::from_env()?;

// Programmatically
let config = AuthConfig::new(
    "your-client-id",
    "your-client-secret",
    "https://your-app.com/oauth/callback",
)
.with_param("additional_param", "value");
```

The following environment variables are used by default:
- `OAUTH_CLIENT_ID`: The client ID for the OAuth2 application
- `OAUTH_CLIENT_SECRET`: The client secret for the OAuth2 application
- `OAUTH_REDIRECT_URI`: The redirect URI for the OAuth2 flow

### Using the Interceptor

The `AuthInterceptor` validates tokens and attaches user information to the request context:

```rust
// Create an interceptor that requires authentication
let auth_interceptor = AuthInterceptor::new(auth_provider);

// Or, create an interceptor that makes authentication optional
let optional_auth_interceptor = AuthInterceptor::optional(auth_provider);

// Apply it to your service
let service = YourServiceImpl::default()
    .with_interceptor(auth_interceptor);
```

### Accessing User Information in Service Methods

Use the `RequestContextExt` trait to access the user information in your service methods:

```rust
use iam::context::RequestContextExt;

async fn my_service_method(&self, request: Request<MyRequest>) -> Result<Response<MyResponse>, Status> {
    // Get the request context
    let context = request.context();
    
    // Check if the user is authenticated
    if context.is_authenticated() {
        // Get the user information
        let user_info = context.user_info().unwrap();
        println!("Request from user: {}", user_info.email);
    }
    
    // Process the request...
    Ok(Response::new(MyResponse {}))
}
```

### Adding a New OAuth2 Provider

To add support for a new OAuth2 provider, implement the `AuthProvider` trait:

```rust
use async_trait::async_trait;
use iam::auth::{AuthProvider, AuthConfig, TokenInfo};
use iam::error::AuthResult;
use iam::user::UserInfo;

pub struct MyCustomProvider {
    config: AuthConfig,
    // ...
}

impl MyCustomProvider {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AuthProvider for MyCustomProvider {
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo> {
        // Implement token validation
    }

    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<TokenInfo> {
        // Implement token refresh
    }

    fn authorize_url(&self, state: &str, scopes: &[&str]) -> String {
        // Generate authorization URL
    }

    async fn exchange_code(&self, code: &str) -> AuthResult<TokenInfo> {
        // Exchange authorization code for tokens
    }

    async fn get_user_info(&self, token: &str) -> AuthResult<UserInfo> {
        // Retrieve user information
    }
}
```

## Authentication Flow

1. **Generate an Authorization URL**: Use the `authorize_url` method to generate a URL for the OAuth2 authorization flow.

2. **Exchange the Authorization Code**: After the user authorizes your application, exchange the authorization code for tokens using the `exchange_code` method.

3. **Validate Tokens**: Use the `validate_token` method to validate tokens and retrieve user information.

4. **Refresh Tokens**: Use the `refresh_token` method to refresh expired tokens.

## Error Handling

All authentication operations return a `Result<T, AuthError>` where `AuthError` is an enum that represents different types of authentication errors. These errors are automatically converted to appropriate tonic `Status` values when used with the interceptor.

## License

This project is licensed under the MIT License - see the LICENSE file for details.