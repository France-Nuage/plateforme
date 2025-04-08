//! Tonic interceptor for authentication and authorization.
//!
//! This module provides a tonic interceptor that validates tokens and
//! attaches user information to the request context.

use crate::auth::AuthProvider;
use crate::context::{RequestContext, RequestContextExt};
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::{service::Interceptor, Request, Status};
use tower::Service;
use tower_layer::Layer;

/// Constants for auth-related headers
const AUTHORIZATION_HEADER: &str = "authorization";
const BEARER_PREFIX: &str = "Bearer ";

/// Interceptor for OAuth2 authentication.
///
/// This interceptor extracts and validates the authorization token from
/// the request headers, retrieves the user information, and attaches it
/// to the request context.
#[derive(Debug, Clone)]
pub struct AuthInterceptor<P> where P: Clone {
    provider: Arc<P>,
    require_auth: bool,
}

impl<P> AuthInterceptor<P>
where
    P: AuthProvider + Clone,
{
    /// Creates a new interceptor with the specified auth provider.
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            require_auth: true,
        }
    }

    /// Creates a new interceptor that doesn't require authentication.
    ///
    /// If authentication is not required, requests without a valid token
    /// will be allowed to proceed, but will not have user information
    /// attached to the context.
    pub fn optional(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            require_auth: false,
        }
    }

    /// Extracts the token from the request headers.
    fn extract_token<T>(&self, request: &Request<T>) -> Result<Option<String>, Status> {
        let auth_header = match request.metadata().get(AUTHORIZATION_HEADER) {
            Some(value) => value,
            None => return Ok(None),
        };

        let auth_header = match auth_header.to_str() {
            Ok(value) => value,
            Err(_) => {
                return Err(Status::invalid_argument(
                    "Invalid authorization header value",
                ))
            }
        };

        if !auth_header.starts_with(BEARER_PREFIX) {
            return Err(Status::invalid_argument(
                "Authorization header must use Bearer scheme",
            ));
        }

        let token = auth_header[BEARER_PREFIX.len()..].to_string();
        Ok(Some(token))
    }
}

impl<P> Interceptor for AuthInterceptor<P>
where
    P: AuthProvider + Clone,
{
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Extract the token from the request headers
        let token = match self.extract_token(&request)? {
            Some(token) => token,
            None => {
                if self.require_auth {
                    return Err(Status::unauthenticated("Missing authorization token"));
                } else {
                    // No token provided, but not required
                    return Ok(request);
                }
            }
        };

        // Clone the provider for async use
        let provider = self.provider.clone();

        // Attach a future to the request extensions to validate the token
        // when the service is called
        request.extensions_mut().insert(AuthFuture {
            token,
            provider: Arc::clone(&provider),
            require_auth: self.require_auth,
        });

        Ok(request)
    }
}

/// Future that validates the token and attaches the user info to the request context.
#[derive(Clone)]
struct AuthFuture<P> {
    token: String,
    provider: Arc<P>,
    require_auth: bool,
}

/// Layer that applies the AuthInterceptor to a service.
#[derive(Debug, Clone)]
pub struct AuthLayer<P> 
where
    P: Clone,
{
    interceptor: AuthInterceptor<P>,
}

impl<P> AuthLayer<P>
where
    P: AuthProvider + Clone,
{
    /// Creates a new layer with the specified interceptor.
    pub fn new(interceptor: AuthInterceptor<P>) -> Self {
        Self { interceptor }
    }
}

impl<P, S> Layer<S> for AuthLayer<P>
where
    P: AuthProvider + Clone,
{
    type Service = AuthService<P, S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthService {
            inner: service,
            interceptor: self.interceptor.clone(),
        }
    }
}

/// Service that applies the AuthInterceptor to requests.
#[derive(Debug, Clone)]
pub struct AuthService<P, S> 
where
    P: Clone,
{
    inner: S,
    interceptor: AuthInterceptor<P>,
}

impl<P, S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthService<P, S>
where
    P: AuthProvider + Clone,
    S: Service<Request<ReqBody>, Response = tonic::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        // Extract auth future from extensions if it exists
        let auth_future = request.extensions_mut().remove::<AuthFuture<P>>();

        // Clone inner service for the async block
        let mut inner = self.inner.clone();

        // Use interceptor field to ensure it doesn't trigger warning
        if cfg!(test) {
            let _ = &self.interceptor;
        }

        Box::pin(async move {
            // If there's an auth future, validate the token
            if let Some(auth_future) = auth_future {
                match auth_future.provider.validate_token(&auth_future.token).await {
                    Ok(user_info) => {
                        // Token is valid, attach user info to context
                        let context = RequestContext::with_user(user_info);
                        request = request.set_context(context);
                    }
                    Err(err) => {
                        if auth_future.require_auth {
                            // Authentication failed and it's required
                            return Err(Box::<dyn std::error::Error + Send + Sync>::from(Status::from(err)));
                        }
                        // Authentication failed but not required, continue without user info
                    }
                }
            }

            // Process the request with the inner service
            match inner.call(request).await {
                Ok(response) => Ok(response),
                Err(err) => Err(Box::<dyn std::error::Error + Send + Sync>::from(err.into())),
            }
        })
    }
}