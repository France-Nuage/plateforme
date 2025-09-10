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
//! ## Per-Request Security Model
//!
//! Each HTTP request receives its own distinct IAM instance created by the authentication
//! middleware. This design ensures complete token isolation between concurrent requests:
//! - **No Token Sharing**: Each IAM instance holds exactly one request's token
//! - **Request Isolation**: Token validation results cannot leak between requests
//! - **Concurrent Safety**: Multiple requests are processed with independent IAM contexts
//!
//! ## Usage Pattern
//!
//! IAM contexts are typically created by authentication middleware and accessed
//! by downstream services through request extensions. Services can call
//! `is_authenticated()` to verify the request's authentication status before
//! processing protected operations.

use sqlx::Postgres;
use tokio::sync::OnceCell;

use crate::{Error, OpenID, model::User, rfc7519::Claim};

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
/// ## Security Model
///
/// Each IAM instance is created per-request by authentication middleware and contains
/// exactly one request's JWT token. This design prevents token leakage between
/// concurrent requests while allowing safe concurrent access within a request's
/// processing pipeline.
///
/// ## Thread Safety
///
/// IAM instances are thread-safe and designed for concurrent access within a single
/// request's processing pipeline. Each request receives its own IAM instance from
/// authentication middleware, preventing token sharing between different requests.
/// The internal OpenID provider handles concurrent token validation efficiently across
/// all requests with shared caching for JWK keys (not tokens or claims).
#[derive(Clone)]
pub struct IAM {
    /// Lazily-loaded JWT claims cache with request-scoped initialization protection.
    ///
    /// Claims are fetched on-demand rather than eagerly during IAM creation to avoid
    /// unnecessary network requests to the OIDC provider. The OnceCell ensures that
    /// token validation occurs only once per request, even when multiple concurrent
    /// calls to claim-dependent methods are made within the same request's processing.
    claim: OnceCell<Claim>,

    openid: OpenID,

    /// Optional JWT token extracted from the Authorization header
    token: Option<String>,
}

impl IAM {
    /// Creates a new IAM context with the provided token and OpenID provider.
    ///
    /// # Arguments
    ///
    /// * `token` - Optional JWT token string extracted from request headers.
    ///   `None` indicates no authentication token was present in the request.
    /// * `openid` - OpenID provider configured with OIDC information
    ///   for token validation
    pub fn new(token: Option<String>, openid: OpenID) -> Self {
        IAM {
            claim: OnceCell::new(),
            openid,
            token,
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
    /// * Other JWT validation errors from the underlying OpenID provider
    ///
    /// # Examples
    ///
    /// ```
    /// # use sqlx::PgPool;
    /// # use auth::{IAM, OpenID};
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
        self.openid.validate_token(&token).await.map(|_| true)
    }

    /// Fetches and validates JWT claims from the OIDC provider.
    ///
    /// This method performs the actual token validation by calling the OpenID provider,
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

        self.openid
            .validate_token(token)
            .await
            .map(|data| data.claims)
    }

    /// Retrieves JWT claims with lazy loading and concurrent initialization protection.
    ///
    /// This method implements a lazy loading pattern for JWT claims validation.
    /// On first access, it validates the token and caches the claims. Subsequent
    /// calls return the cached claims without re-validation. The implementation
    /// uses OnceCell to ensure token validation occurs exactly once per request.
    ///
    /// # Concurrency Behavior
    ///
    /// - **First Call**: Initializes OnceCell with validated token claims
    /// - **Subsequent Calls**: Returns cached claims from OnceCell immediately  
    /// - **Concurrent Calls**: OnceCell ensures only one validation occurs, others wait for result
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
        self.claim
            .get_or_try_init(|| async { self.fetch_claim().await })
            .await
            .cloned()
    }
}
