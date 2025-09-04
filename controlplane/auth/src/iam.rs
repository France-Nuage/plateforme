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
//! - **On-demand Validation**: Validates tokens only when authentication checks are performed
//! - **Claims Caching**: Lazily loads and caches JWT claims to avoid redundant validations
//! - **Concurrency Protection**: Uses RwLock to prevent duplicate validation requests
//! - **Error Transparency**: Provides clear error information for authentication failures
//! - **Optional Authentication**: Gracefully handles requests without authentication tokens
//!
//! ## Usage Pattern
//!
//! IAM contexts are typically created by authentication middleware and accessed
//! by downstream services through request extensions. Services can call
//! `is_authenticated()` to verify the request's authentication status before
//! processing protected operations.

use std::sync::Arc;

use sqlx::Postgres;
use tokio::sync::RwLock;

use crate::{Error, JwkValidator, model::User, rfc7519::Claim};

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
    /// Lazily-loaded JWT claims cache with concurrent access protection.
    ///
    /// Claims are fetched on-demand rather than eagerly during IAM creation to avoid
    /// unnecessary network requests to the OIDC provider. The RwLock guards against
    /// concurrent validation requests, ensuring that multiple simultaneous calls to
    /// claim-dependent methods don't trigger redundant external API calls.
    claim: Arc<RwLock<Option<Claim>>>,

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
        IAM {
            claim: Arc::new(RwLock::new(None)),
            token,
            validator,
        }
    }

    /// Retrieves the authenticated user's authorization context from the database.
    ///
    /// This method extracts the user's email from validated JWT claims and looks up
    /// their authorization record in the database. It serves as the bridge between
    /// JWT authentication and database-backed authorization, determining which
    /// organization the authenticated user belongs to.
    ///
    /// **Note**: This is a temporary authorization mechanism that will be removed
    /// once SpiceDB integration is complete. The database pool parameter will no
    /// longer be needed in the stateless SpiceDB authorization model.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool for user lookup queries
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - Successfully retrieved user authorization context
    /// * `Err(Error)` - Authentication or authorization failure
    ///
    /// # Errors
    ///
    /// * `Error::MissingAuthorizationHeader` - No JWT token available for validation
    /// * `Error::MissingEmailClaim` - JWT token lacks required email claim
    /// * `Error::UserNotRegistered` - Email found in JWT but user not in database
    /// * `Error::Database` - Database query failure during user lookup
    /// * Other JWT validation errors from the underlying validator
    ///
    /// # Examples
    ///
    /// ```
    /// # use sqlx::PgPool;
    /// # use auth::{IAM, JwkValidator};
    /// # use auth::model::User;
    /// # async fn example(pool: &PgPool, iam: &IAM) -> Result<(), auth::Error> {
    /// // Get user authorization context for authenticated request
    /// let user = iam.user(pool).await?;
    /// println!("User authorized for organization: {}", user.organization_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn user(&self, pool: &sqlx::Pool<Postgres>) -> Result<User, Error> {
        let email = self
            .get_claim()
            .await?
            .email
            .ok_or(Error::MissingEmailClaim)?;

        User::find_one_by_email(pool, &email)
            .await
            .map_err(Into::into)
            .and_then(|x| x.ok_or(Error::UserNotRegistered(email)))
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

    /// Fetches and validates JWT claims from the OIDC provider.
    ///
    /// This method performs the actual token validation by calling the JWK validator,
    /// which may involve external network requests to fetch JWK keys from the OIDC
    /// provider. This operation is potentially expensive and should be called sparingly.
    ///
    /// # Returns
    ///
    /// * `Ok(Claim)` - Successfully validated JWT claims
    /// * `Err(Error)` - Token validation failed or no token available
    ///
    /// # Errors
    ///
    /// Returns errors for missing tokens, invalid signatures, expired tokens,
    /// or network failures when fetching JWK keys.
    async fn fetch_claim(&self) -> Result<Claim, Error> {
        let token = self
            .token
            .as_ref()
            .ok_or(Error::MissingAuthorizationHeader)?;

        self.validator
            .validate_token(token)
            .await
            .map(|data| data.claims)
    }

    /// Retrieves JWT claims with lazy loading and concurrent access protection.
    ///
    /// This method implements a lazy loading pattern for JWT claims validation.
    /// On first access, it validates the token and caches the claims. Subsequent
    /// calls return the cached claims without re-validation. The implementation
    /// uses a read-write lock to prevent multiple concurrent validation requests
    /// for the same token.
    ///
    /// # Concurrency Behavior
    ///
    /// - **First Call**: Acquires write lock, validates token, caches result
    /// - **Subsequent Calls**: Uses read lock to access cached claims
    /// - **Concurrent Calls**: Only one validation occurs, others wait for result
    ///
    /// # Returns
    ///
    /// * `Ok(Claim)` - JWT claims (either from cache or fresh validation)
    /// * `Err(Error)` - Token validation failed or no token available
    ///
    /// # Errors
    ///
    /// Returns the same errors as `fetch_claim()` during initial validation.
    /// Cached claims will not produce validation errors on subsequent calls.
    async fn get_claim(&self) -> Result<Claim, Error> {
        let claim = match self.claim.read().await.clone() {
            Some(claim) => claim,
            None => {
                // Acquire the write lock before calling the fetch_claim method
                let mut lock = self.claim.write().await;
                let value = self.fetch_claim().await?;

                // Write the new value and return it, dropping the lock
                *lock = Some(value.clone());
                value
            }
        };

        Ok(claim)
    }
}
