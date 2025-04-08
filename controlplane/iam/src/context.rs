//! Request context for gRPC services.
//! 
//! This module provides a way to attach user information to gRPC requests
//! and access it from service handlers.

use crate::user::UserInfo;
use std::sync::Arc;

/// Context that holds user information for a request.
///
/// This struct is used to attach user information to gRPC requests
/// and propagate it to service handlers.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// The authenticated user information.
    user_info: Option<Arc<UserInfo>>,
}

impl RequestContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self { user_info: None }
    }

    /// Creates a new context with user information.
    pub fn with_user(user_info: UserInfo) -> Self {
        Self {
            user_info: Some(Arc::new(user_info)),
        }
    }

    /// Returns the user information, if available.
    pub fn user_info(&self) -> Option<&UserInfo> {
        self.user_info.as_deref()
    }

    /// Returns whether the request has an authenticated user.
    pub fn is_authenticated(&self) -> bool {
        self.user_info.is_some()
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for tonic::Request that provides access to the request context.
pub trait RequestContextExt<T> {
    /// Gets the request context, creating an empty one if it doesn't exist.
    fn context(&self) -> RequestContext;

    /// Sets the request context.
    fn set_context(self, context: RequestContext) -> tonic::Request<T>;
}

impl<T> RequestContextExt<T> for tonic::Request<T> {
    fn context(&self) -> RequestContext {
        self.extensions()
            .get::<RequestContext>()
            .cloned()
            .unwrap_or_default()
    }

    fn set_context(mut self, context: RequestContext) -> tonic::Request<T> {
        self.extensions_mut().insert(context);
        self
    }
}