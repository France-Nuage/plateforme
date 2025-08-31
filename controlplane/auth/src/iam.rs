//! Identity and Access Management (IAM) context for authenticated requests.
//!
//! This module provides the `IAM` struct which serves as an authentication context
//! for individual HTTP requests. The IAM context contains the JWT token (if present)
//! and provides methods to validate authentication status using OIDC JWT validation.
//!
//! ## Design Philosophy
//!
//! The IAM context follows a lazy validation approach:
//! - **Token Storage**: Stores the raw JWT token without immediate validation
//! - **On-demand Validation**: Validates tokens only when `is_authenticated()` is called
//! - **Error Transparency**: Provides clear error information for authentication failures
//! - **Optional Authentication**: Gracefully handles requests without authentication tokens
//!
//! ## Usage Pattern
//!
//! IAM contexts are typically created by authentication middleware and accessed
//! by downstream services through request extensions. Services can call
//! `is_authenticated()` to verify the request's authentication status before
//! processing protected operations.

use crate::{Error, JwkValidator};

/// Identity and Access Management context for HTTP request authentication.
///
/// `IAM` provides a per-request authentication context that holds JWT tokens
/// and offers methods to validate authentication status. This struct is designed
/// to be injected into HTTP request extensions by authentication middleware and
/// accessed by downstream services.
///
/// ## Authentication Flow
///
/// 1. **Creation**: IAM is created by authentication middleware with extracted token
/// 2. **Storage**: Token is stored without immediate validation for performance
/// 3. **Validation**: Token validation occurs on-demand when `is_authenticated()` is called
/// 4. **Caching**: Validation results could be cached within the request scope (future enhancement)
///
/// ## Thread Safety
///
/// IAM is thread-safe and can be cloned across async tasks. The internal validator
/// handles concurrent token validation efficiently with its own caching mechanisms.
#[derive(Clone)]
pub struct IAM {
    /// Optional JWT token extracted from the Authorization header
    token: Option<String>,
    /// JWK validator for performing JWT token validation
    validator: JwkValidator,
}

impl IAM {
    /// Creates a new IAM context with the provided token and validator.
    ///
    /// # Arguments
    ///
    /// * `token` - Optional JWT token string extracted from request headers.
    ///   `None` indicates no authentication token was present in the request.
    /// * `validator` - JWK validator configured with OIDC provider information
    ///   for token validation
    pub fn new(token: Option<String>, validator: JwkValidator) -> Self {
        IAM { token, validator }
    }

    /// Validates the authentication status of the current request.
    ///
    /// This method performs JWT token validation using the configured OIDC provider's
    /// JWK keys. It returns `true` if a valid JWT token is present and successfully
    /// validated, `false` otherwise.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Request has a valid, authenticated JWT token
    /// * `Err(Error)` - Authentication failed or no token present
    ///
    /// # Errors
    ///
    /// This method returns errors in the following cases:
    /// - `MissingAuthorizationHeader` - No JWT token was provided in the request
    /// - JWT validation errors - Token signature, expiration, or format issues
    /// - Network errors - Problems fetching JWK keys from OIDC provider
    pub async fn is_authenticated(&self) -> Result<bool, Error> {
        let token = self
            .token
            .clone()
            .ok_or(Error::MissingAuthorizationHeader)?;
        self.validator.validate_token(&token).await.map(|_| true)
    }
}
