//! Authentication middleware layer for HTTP requests.
//!
//! This module provides Tower middleware for HTTP request authentication using OIDC JWT tokens.
//! The middleware automatically extracts JWT tokens from request headers, validates them using
//! JWK validation, and injects an IAM (Identity and Access Management) context into request
//! extensions for downstream services.
//!
//! ## Key Features
//!
//! - **Token Extraction**: Automatically extracts JWT tokens from `Authorization` headers
//! - **Non-blocking**: Processes authentication asynchronously without blocking request flow
//! - **Context Injection**: Provides IAM context to all downstream request handlers
//! - **Standards Compliant**: Follows RFC 6750 Bearer token authentication specification
//!
//! ## Middleware Flow
//!
//! 1. **Header Extraction**: Extract `authorization` header value from incoming request
//! 2. **Token Parsing**: Parse Bearer token from header (if present)  
//! 3. **IAM Creation**: Create IAM context with extracted token and OpenID provider
//! 4. **Context Injection**: Insert IAM into request extensions for downstream access
//! 5. **Request Forwarding**: Forward request to inner service with injected context
//!
//! ## Usage Pattern
//!
//! The authentication layer is typically applied as middleware in a Tower service stack:
//!
//! ```
//! use auth::{AuthenticationLayer, OpenID};
//! use tower::ServiceBuilder;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let openid = OpenID::discover(reqwest::Client::new(), "https://provider.com/.well-known/openid_configuration").await?;
//! let auth_layer = AuthenticationLayer::new(openid);
//!
//! let service = ServiceBuilder::new().layer(auth_layer);
//! # Ok(())
//! # }
//! ```

use crate::{iam::IAM, openid::OpenID};
use http::Request;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Tower middleware layer for JWT authentication with OIDC validation.
///
/// `AuthenticationLayer` implements the Tower `Layer` trait to provide HTTP request
/// authentication middleware. It uses an OpenID provider to handle JWT token validation
/// and injects IAM context into each request for downstream services.
///
/// ## Design
///
/// The layer follows Tower's middleware pattern:
/// - **Layer**: Creates new `AuthenticationService` instances  
/// - **Service**: Processes individual requests with authentication logic
/// - **Cloneable**: Supports Tower's service cloning requirements for concurrent request handling
///
/// ## Thread Safety
///
/// This layer is thread-safe and can be safely cloned across multiple tokio tasks.
/// The internal `OpenID` handles concurrent JWT validation efficiently.
#[derive(Clone)]
pub struct AuthenticationLayer {
    openid: OpenID,
}

impl AuthenticationLayer {
    /// Creates a new authentication layer with the provided OpenID provider.
    ///
    /// # Arguments
    ///
    /// * `openid` - A configured OpenID provider that will be used to validate
    ///   JWT tokens extracted from request headers
    ///
    /// # Examples
    ///
    /// ```
    /// use auth::{AuthenticationLayer, OpenID};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let openid = OpenID::discover(
    ///     reqwest::Client::new(),
    ///     "https://accounts.google.com/.well-known/openid_configuration"
    /// ).await?;
    ///
    /// let auth_layer = AuthenticationLayer::new(openid);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(openid: OpenID) -> Self {
        Self { openid }
    }
}

impl<S> Layer<S> for AuthenticationLayer {
    type Service = AuthenticationService<S>;

    /// Creates a new authentication service wrapping the provided inner service.
    ///
    /// This method is called by Tower's middleware system to wrap services with
    /// authentication functionality. The returned service will process all requests
    /// through the authentication flow before forwarding to the inner service.
    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationService {
            inner,
            openid: self.openid.clone(),
        }
    }
}

/// Tower service that provides JWT authentication for HTTP requests.
///
/// `AuthenticationService` wraps an inner service and processes all incoming requests
/// to extract JWT tokens, create IAM contexts, and inject authentication information
/// into request extensions. This service is created by `AuthenticationLayer` and
/// handles the actual authentication logic for each request.
///
/// ## Request Processing
///
/// For each incoming request, this service:
/// 1. Extracts the `authorization` header value
/// 2. Parses any Bearer token present in the header
/// 3. Creates an IAM context with the token and OpenID provider
/// 4. Injects the IAM context into request extensions
/// 5. Forwards the request to the inner service
///
/// ## Security Isolation
///
/// This service creates a new IAM instance for every incoming request, ensuring
/// complete token isolation between concurrent requests. Each request's authentication
/// context is independent and cannot access tokens from other requests.
///
/// ## Error Handling
///
/// This service does not fail requests that lack authentication headers.
/// Instead, it creates an IAM context with `None` token, allowing downstream
/// services to decide how to handle unauthenticated requests.
#[derive(Clone)]
pub struct AuthenticationService<S> {
    /// The inner service that will receive authenticated requests
    inner: S,

    openid: OpenID,
}

impl<S, ReqBody> Service<Request<ReqBody>> for AuthenticationService<S>
where
    S: Service<Request<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Processes an incoming HTTP request with authentication.
    ///
    /// This method extracts JWT tokens from the `authorization` header, creates
    /// an IAM context, and injects it into the request extensions before forwarding
    /// the request to the inner service.
    ///
    /// ## Header Processing
    ///
    /// The method looks for an `authorization` header and extracts its value if present.
    /// No Bearer token parsing is performed at this level - the raw header value is
    /// passed to the IAM context for later processing.
    ///
    /// ## Context Injection
    ///
    /// An `IAM` instance is always created and inserted into request extensions,
    /// even when no authorization header is present. This ensures downstream
    /// services can consistently access authentication context.
    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let token = req
            .headers()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_owned());

        let iam = IAM::new(token, self.openid.clone());
        req.extensions_mut().insert(iam);

        self.inner.call(req)
    }
}
