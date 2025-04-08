//! # IAM - Identity and Access Management
//!
//! This crate provides OAuth2 authentication and authorization capabilities
//! for gRPC services. It is designed to be provider-agnostic with a clean
//! abstraction layer that supports multiple OAuth2 providers.
//!
//! ## Features
//!
//! - Provider-agnostic OAuth2 authentication system
//! - Support for multiple OAuth2 providers (Google implemented by default)
//! - Tonic interceptor for validating tokens and attaching user info to request context
//! - Async/await for non-blocking operations
//! - Comprehensive error handling
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use iam::{
//!     auth::AuthConfig,
//!     interceptor::AuthInterceptor,
//!     providers::google::GoogleAuthProvider,
//! };
//! use tonic::transport::Server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure the auth provider
//!     let auth_config = AuthConfig::from_env()?;
//!     let auth_provider = GoogleAuthProvider::new(auth_config);
//!
//!     // Create an interceptor with the auth provider
//!     let auth_interceptor = AuthInterceptor::new(auth_provider);
//!
//!     // Use the interceptor with your gRPC service
//!     // In your real code, replace this with your actual service
//!     let service = tonic::service::service_fn(|_| async {
//!         Ok::<_, tonic::Status>(tonic::Response::new(()))
//!     });
//!
//!     // Start the server
//!     Server::builder()
//!         .add_service(service)
//!         .serve("0.0.0.0:50051".parse()?)
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod context;
pub mod error;
pub mod interceptor;
pub mod providers;
pub mod user;
#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use auth::{AuthConfig, AuthProvider};
pub use context::RequestContext;
pub use error::AuthError;
pub use interceptor::AuthInterceptor;
pub use user::UserInfo;