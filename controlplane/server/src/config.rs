//! Server configuration management for the gRPC server.
//!
//! This module provides the [`Config`] structure that encapsulates all
//! configuration parameters needed for server operation, including network
//! settings, CORS policies, OIDC authentication, and PostgreSQL database connectivity.
//! The configuration system provides sensible defaults suitable for development while
//! remaining flexible for production deployments.

use crate::error::Error;
use auth::OpenID;
use mock_server::MockServer;
use sqlx::{Pool, Postgres};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, ExposeHeaders};

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
/// use server::config::Config;
/// use sqlx::PgPool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://localhost/db").await?;
/// # let mock = mock_server::MockServer::new().await;
/// let config = Config::test(&pool, &mock).await?;
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

    /// CORS configuration specifying which headers are allowed in cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Headers` header in HTTP responses.
    /// The default configuration allows all headers using [`AllowHeaders::any()`].
    ///
    /// [`AllowHeaders::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowHeaders.html#method.any
    pub allow_headers: AllowHeaders,

    /// CORS configuration specifying which HTTP methods are allowed for cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Methods` header in HTTP responses.
    /// The default configuration allows all HTTP methods using [`AllowMethods::any()`].
    ///
    /// [`AllowMethods::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowMethods.html#method.any
    pub allow_methods: AllowMethods,

    /// CORS configuration specifying which origins are allowed to make cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Origin` header in HTTP responses.
    /// The default configuration allows requests from any origin using [`AllowOrigin::any()`].
    ///
    /// [`AllowOrigin::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowOrigin.html#method.any
    pub allow_origin: AllowOrigin,

    /// CORS configuration specifying which response headers are exposed to client scripts.
    ///
    /// This field controls the `Access-Control-Expose-Headers` header in HTTP responses.
    /// The default configuration exposes all headers using [`ExposeHeaders::any()`].
    ///
    /// [`ExposeHeaders::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.ExposeHeaders.html#method.any
    pub expose_headers: ExposeHeaders,

    /// PostgreSQL database connection pool for persistent storage operations.
    ///
    /// This field provides the PostgreSQL connection pool that will be shared across
    /// all services for performing persistent storage operations.
    pub pool: Pool<Postgres>,

    pub openid: OpenID,
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
    /// - **OpenID Provider**: Uses the provided OpenID provider for OIDC authentication
    ///
    /// # Parameters
    ///
    /// * `pool` - PostgreSQL connection pool that will be shared across all services
    /// * `openid` - OpenID provider configured with OIDC information for JWT authentication
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use server::config::Config;
    /// use sqlx::PgPool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://localhost/db").await?;
    /// # let mock = mock_server::MockServer::new().await;
    /// let config = Config::test(&pool, &mock).await?;
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

    /// Creates a test configuration with a dynamically allocated port and mock OIDC server.
    ///
    /// This constructor is specifically designed for test environments where:
    /// - A random available port is automatically allocated to avoid conflicts
    /// - OIDC authentication is configured to use the provided mock server
    /// - Database connection pool is cloned from the provided reference
    ///
    /// ## Parameters
    ///
    /// * `pool` - Reference to PostgreSQL connection pool (will be cloned)
    /// * `mock_server` - Mock server instance for OIDC authentication testing
    ///
    /// ## Usage in Tests
    ///
    /// ```
    /// # use server::Config;
    /// # use mock_server::MockServer;
    /// # async fn example(pool: &sqlx::PgPool) -> Result<(), server::error::Error> {
    /// let mock = MockServer::new().await;
    /// let config = Config::test(pool, &mock).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Features
    ///
    /// - **Dynamic Port**: Uses `reserve_socket_addr(None)` to allocate an available port
    /// - **Mock Authentication**: Configures OpenID for the mock server
    /// - **Test Isolation**: Each test gets its own port to avoid interference
    pub async fn test(pool: &Pool<Postgres>, mock_server: &MockServer) -> Result<Self, Error> {
        let addr = Config::reserve_socket_addr(None).await?;

        let client = reqwest::Client::new();
        let openid = OpenID::discover(
            client,
            &format!("{}/.well-known/openid-configuration", &mock_server.url()),
        )
        .await?;

        Ok(Config {
            addr,
            allow_headers: AllowHeaders::any(),
            allow_methods: AllowMethods::any(),
            allow_origin: AllowOrigin::any(),
            expose_headers: ExposeHeaders::any(),
            pool: pool.clone(),
            openid,
        })
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

        let client = reqwest::Client::new();
        let openid = OpenID::discover(client, &oidc_url)
            .await
            .expect("could not fetch oidc configuration");

        Ok(Config {
            addr: Config::reserve_socket_addr(env::var("CONTROLPLANE_ADDR").ok()).await?,
            allow_headers: AllowHeaders::any(),
            allow_methods: AllowMethods::any(),
            allow_origin: AllowOrigin::any(),
            expose_headers: ExposeHeaders::any(),
            pool,
            openid,
        })
    }

    /// Reserves a socket address, either from a preset string or by allocating dynamically.
    ///
    /// This method provides flexible address allocation for server binding:
    /// - If a preset address is provided, it parses and validates the address
    /// - If no preset is given, it allocates an available port on the loopback interface
    ///
    /// ## Parameters
    ///
    /// * `preset` - Optional address string (e.g., "127.0.0.1:8080", "[::1]:3000")
    ///
    /// ## Returns
    ///
    /// Returns a `SocketAddr` that can be used for server binding.
    ///
    /// ## Behavior
    ///
    /// - **With preset**: Parses the provided address string
    /// - **Without preset**: Binds to `[::1]:0` to get an OS-allocated port
    ///
    /// ## Usage
    ///
    /// ```
    /// # use server::Config;
    /// # async fn example() -> Result<(), server::error::Error> {
    /// // Use specific address
    /// let addr1 = Config::reserve_socket_addr(Some("127.0.0.1:8080".to_string())).await?;
    ///
    /// // Allocate dynamic port
    /// let addr2 = Config::reserve_socket_addr(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reserve_socket_addr(preset: Option<String>) -> Result<SocketAddr, Error> {
        match preset {
            Some(preset) => preset.parse().map_err(Into::into),
            None => TcpListener::bind("[::1]:0")
                .await?
                .local_addr()
                .map_err(Into::into),
        }
    }
}
