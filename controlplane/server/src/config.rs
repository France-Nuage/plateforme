//! Server configuration management for the gRPC server.
//!
//! This module provides the [`Config`] structure that encapsulates all
//! configuration parameters needed for server operation, including network
//! settings, CORS policies, and PostgreSQL database connectivity. The configuration system provides sensible
//! defaults suitable for development while remaining flexible for production
//! deployments.

use std::{net::SocketAddr, str::FromStr};

use sqlx::{Pool, Postgres};
use tower_http::cors::{AllowMethods, AllowOrigin};

/// Configuration for the gRPC server with CORS, networking, and PostgreSQL database settings.
///
/// This structure encapsulates all the necessary configuration parameters for setting up
/// a gRPC server with Cross-Origin Resource Sharing (CORS) support and PostgreSQL database connectivity.
/// It provides sensible defaults suitable for development environments while maintaining
/// flexibility for production deployments.
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
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL connection pool that will be shared across all services
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
    pub fn new(pool: Pool<Postgres>) -> Self {
        Config {
            addr: SocketAddr::from_str("[::]:8080").unwrap(),
            allow_origin: AllowOrigin::any(),
            allow_methods: AllowMethods::any(),
            pool,
        }
    }
}
