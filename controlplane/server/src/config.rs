//! Server configuration management for the gRPC server.
//!
//! This module provides the [`Config`] structure that encapsulates all
//! configuration parameters needed for server operation, including network
//! settings, CORS policies, OIDC authentication, and PostgreSQL database connectivity.
//! The configuration system provides sensible defaults suitable for development while
//! remaining flexible for production deployments.

use std::{env, net::SocketAddr, str::FromStr};

use auth::JwkValidator;
use sqlx::{Pool, Postgres};
use tower_http::cors::{AllowMethods, AllowOrigin};

use crate::error::Error;

/// Configuration for the gRPC server with CORS, authentication, networking, and PostgreSQL database settings.
///
/// This structure encapsulates all the necessary configuration parameters for setting up
/// a gRPC server with Cross-Origin Resource Sharing (CORS) support, OIDC JWT authentication,
/// and PostgreSQL database connectivity. It provides sensible defaults suitable for
/// development environments while maintaining flexibility for production deployments.
///
/// # Example
///
/// ```rust,no_run
/// use config::Config;
/// use sqlx::PgPool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://localhost/db").await?;
/// let config = Config::new(pool);
/// # Ok(())
/// # }
/// ```
///
/// The default configuration binds to all interfaces on port 8080 and allows any origin
/// and HTTP methods, making it suitable for development scenarios.
#[derive(Clone)]
pub struct Config {
    /// The socket address where the server will bind and listen for connections.
    ///
    /// This field determines both the IP address and port number that the gRPC server
    /// will use. The default value binds to all available interfaces (`[::]`) on port 8080.
    pub addr: SocketAddr,

    /// CORS configuration specifying which origins are allowed to make cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Origin` header in HTTP responses.
    /// The default configuration allows requests from any origin using [`AllowOrigin::any()`].
    ///
    /// [`AllowOrigin::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowOrigin.html#method.any
    pub allow_origin: AllowOrigin,

    /// CORS configuration specifying which HTTP methods are allowed for cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Methods` header in HTTP responses.
    /// The default configuration allows all HTTP methods using [`AllowMethods::any()`].
    ///
    /// [`AllowMethods::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowMethods.html#method.any
    pub allow_methods: AllowMethods,

    /// PostgreSQL database connection pool for persistent storage operations.
    ///
    /// This field provides the PostgreSQL connection pool that will be shared across
    /// all services for performing persistent storage operations.
    pub pool: Pool<Postgres>,

    /// JWT validator for OIDC authentication middleware.
    ///
    /// This field provides the JWK validator that will be used by the authentication
    /// middleware to validate JWT tokens from incoming requests. The validator is
    /// configured with OIDC provider information and handles key caching automatically.
    pub validator: JwkValidator,
}

impl Config {
    /// Creates a new [`Config`] instance with development-friendly defaults.
    ///
    /// This constructor initializes the configuration with settings appropriate for
    /// local development and testing environments:
    ///
    /// - **Address**: `[::]:8080` - Binds to all interfaces on port 8080
    /// - **Allow Origin**: [`AllowOrigin::any()`] - Accepts requests from any origin
    /// - **Allow Methods**: [`AllowMethods::any()`] - Permits all HTTP methods
    /// - **PostgreSQL Pool**: Uses the provided connection pool for database operations
    /// - **JWT Validator**: Uses the provided validator for OIDC authentication
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL connection pool that will be shared across all services
    /// * `validator` - JWK validator configured with OIDC provider for JWT authentication
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use config::Config;
    /// use sqlx::PgPool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://localhost/db").await?;
    /// let config = Config::new(pool);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// The default configuration is permissive and intended for development use.
    /// For production deployments, consider creating more restrictive configurations
    /// by constructing the struct manually or adding additional constructor methods.
    ///
    /// [`AllowOrigin::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowOrigin.html#method.any
    /// [`AllowMethods::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowMethods.html#method.any
    pub fn new(pool: Pool<Postgres>, validator: JwkValidator) -> Self {
        Config {
            addr: SocketAddr::from_str("[::]:8080").unwrap(),
            allow_origin: AllowOrigin::any(),
            allow_methods: AllowMethods::any(),
            pool,
            validator,
        }
    }

    /// Creates a configuration instance from environment variables.
    ///
    /// This method provides a convenient way to initialize server configuration
    /// from environment variables, making it suitable for containerized deployments
    /// and production environments.
    ///
    /// # Environment Variables
    ///
    /// * `DATABASE_URL` - PostgreSQL connection string (required)
    /// * `OIDC_URL` - OIDC provider discovery URL (optional, defaults to GitLab)
    ///
    /// # Default Values
    ///
    /// - **OIDC Provider**: GitLab's OIDC discovery endpoint if `OIDC_URL` not set
    /// - **Server Settings**: Same defaults as [`Config::new()`] for networking and CORS
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `DATABASE_URL` environment variable is not set
    /// - Database connection cannot be established
    /// - OIDC discovery fails or provider is unreachable
    /// - OIDC provider configuration is invalid
    pub async fn from_env() -> Result<Self, Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("could not connect to database");
        let oidc_url = env::var("OIDC_URL").unwrap_or(String::from(
            "https://gitlab.com/.well-known/openid-configuration",
        ));
        let validator = JwkValidator::from_oidc_discovery(&oidc_url)
            .await
            .expect("could not fetch oidc configuration");
        let config = Config::new(pool, validator);

        Ok(config)
    }
}
