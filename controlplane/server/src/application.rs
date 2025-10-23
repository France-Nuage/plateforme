//! Application orchestration and composition for the gRPC server.
//!
//! This module provides the [`Application`] structure that serves as the main
//! orchestrator for composing a complete gRPC server application. It
//! implements a builder pattern that allows progressive configuration of
//! middleware layers, service registration, and dependency injection.
//!
//! The application structure encapsulates all major components needed for a
//! production-ready gRPC server: configuration management, PostgreSQL database
//! connectivity, request routing, and server middleware stack.

use crate::config::Config;
use crate::error::Error;
use crate::router::Router;
use crate::server::{Server, TraceLayer};
use auth::AuthenticationLayer;
use std::future::Future;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_web::GrpcWebLayer;
use tower::layer::util::{Identity, Stack};
use tower_http::cors::CorsLayer;

/// Main application structure that orchestrates the gRPC server components.
///
/// This structure provides a builder pattern for composing a complete gRPC
/// application with PostgreSQL database connectivity, middleware support, and service
/// registration. It encapsulates the configuration, database connection pool,
/// routing logic, and server instance into a cohesive unit that can be
/// progressively configured and then executed.
///
/// # Type Parameters
///
/// * `L` - The layer type for middleware stack composition, starting with
///   [`Identity`]
///
/// # Example
///
/// ```rust,no_run
/// use server::application::Application;
/// use server::config::Config;
/// use sqlx::PgPool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://localhost/db").await?;
/// # let mock = mock_server::MockServer::new().await;
/// let config = Config::test(&pool, &mock).await?;
///
/// let app = Application::new(config)
///     .with_middlewares()
///     .with_services();
/// # Ok(())
/// # }
/// ```
///
/// [`Identity`]: https://docs.rs/tower-layer/latest/tower_layer/struct.Identity.html
pub struct Application<L> {
    /// Server configuration including network settings and CORS policies.
    config: Config,
    /// Request routing and service registration handler.
    router: Router,
    /// HTTP/gRPC server instance with middleware layers.
    server: Server<L>,
}

impl Application<Identity> {
    /// Creates a new [`Application`] instance with the provided configuration.
    ///
    /// This constructor initializes the application with minimal dependencies,
    /// creating new instances of [`Router`] and [`Server`] with their default
    /// configurations. The resulting application has no middleware layers
    /// ([`Identity`] layer) and no registered services, making it ready for
    /// progressive configuration through the builder pattern.
    ///
    /// # Parameters
    ///
    /// * `config` - Server configuration settings including PostgreSQL connection pool
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use server::application::Application;
    /// use server::config::Config;
    /// use sqlx::PgPool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgresql://localhost/db").await?;
    /// # let mock = mock_server::MockServer::new().await;
    /// let config = Config::test(&pool, &mock).await?;
    ///
    /// let app = Application::new(config);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Next Steps
    ///
    /// After creating the application, you typically want to add middleware
    /// and services:
    ///
    /// ```rust,no_run
    /// # use server::application::Application;
    /// # use server::config::Config;
    /// # use sqlx::PgPool;
    /// # async fn example(config: Config) {
    /// let app = Application::new(config)
    ///     .with_middlewares()  // Add CORS and other middleware
    ///     .with_services();    // Register gRPC services
    /// # }
    /// ```
    pub fn new(config: Config) -> Self {
        Self {
            config,
            router: Router::new(),
            server: Server::new(),
        }
    }
}

/// Type alias for the complete middleware stack composition.
///
/// This represents the full middleware stack that will be applied to the
/// server, composed on top of the existing layer `L`.
type Middleware<L> =
    Stack<AuthenticationLayer, Stack<GrpcWebLayer, Stack<CorsLayer, Stack<TraceLayer, L>>>>;

impl<L> Application<L> {
    /// Adds the complete middleware stack to the application server.
    ///
    /// This method applies all configured middleware layers to the server,
    /// creating a production-ready middleware stack for handling cross-cutting
    /// concerns like security, observability, and request/response processing.
    ///
    /// # Enabled Middleware
    ///
    /// The following middleware layers are applied in order:
    /// - **Authentication**: OIDC JWT token validation middleware that validates
    ///   Bearer tokens in request metadata and rejects unauthenticated requests
    /// - **CORS**: Cross-Origin Resource Sharing support using configuration
    ///   settings for allowed origins and methods
    /// - **Tracing**: Request tracing and observability middleware
    ///
    /// # Type Transformation
    ///
    /// This method transforms the application from `Application<L, DB>`
    /// to `Application<Middleware<L>, DB>`, where [`Middleware<L>`]
    /// represents the complete middleware stack.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use server::application::Application;
    /// use server::config::Config;
    /// use sqlx::PgPool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = PgPool::connect("postgresql://localhost/db").await?;
    /// # let mock = mock_server::MockServer::new().await;
    /// # let config = Config::test(&pool, &mock).await?;
    /// let app = Application::new(config)
    ///     .with_middlewares(); // Applies all middleware layers
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_middlewares(self) -> Application<Middleware<L>> {
        Application {
            config: self.config.clone(),
            router: self.router,
            server: self
                .server
                .with_tracing()
                .with_cors(
                    self.config.allow_headers,
                    self.config.allow_methods,
                    self.config.allow_origin,
                    self.config.expose_headers,
                )
                .with_web()
                .with_authentication(self.config.authz, self.config.openid),
        }
    }

    /// Registers all gRPC services with the application router.
    ///
    /// This method configures the router with all available gRPC services,
    /// establishing the complete service layer for handling client requests.
    /// Each service is provided with necessary dependencies like PostgreSQL
    /// connection pools for data persistence operations.
    ///
    /// # Registered Services
    ///
    /// The following gRPC services are registered:
    /// - **Instances**: Instance management service for virtual machine
    ///   lifecycle operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use server::application::Application;
    /// use server::config::Config;
    /// use sqlx::PgPool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = PgPool::connect("postgresql://localhost/db").await?;
    /// # let mock = mock_server::MockServer::new().await;
    /// # let config = Config::test(&pool, &mock).await?;
    /// let app = Application::new(config)
    ///     .with_middlewares()
    ///     .with_services(); // Registers all gRPC services
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// Services are registered with shared PostgreSQL pool instances to ensure
    /// efficient connection management across the application.
    pub fn with_services(self) -> Application<L> {
        let iam = self.config.app.iam.clone();
        let pool = self.config.pool.clone();
        let hypervisors = self.config.app.hypervisors.clone();
        let invitations = self.config.app.invitations.clone();
        let organizations = self.config.app.organizations.clone();
        let projects = self.config.app.projects.clone();
        let users = self.config.app.users.clone();
        let zones = self.config.app.zones.clone();
        Self {
            config: self.config,
            router: self
                .router
                .health()
                .hypervisors(iam.clone(), pool.clone(), hypervisors.clone())
                .instances(
                    iam.clone(),
                    pool.clone(),
                    hypervisors.clone(),
                    projects.clone(),
                )
                .invitations(iam.clone(), invitations.clone(), users.clone())
                .reflection()
                .resources(iam.clone(), organizations, pool.clone())
                .zero_trust_networks(pool.clone())
                .zero_trust_network_types(pool.clone())
                .zones(iam.clone(), zones.clone()),

            server: self.server,
        }
    }
}

impl Application<Middleware<Identity>> {
    /// Starts the gRPC server and runs until a shutdown signal is received.
    ///
    /// This method starts the configured gRPC server with all registered
    /// services and middleware, binding to the configured address and listening
    /// for incoming connections. The server will continue running until the
    /// provided shutdown signal future completes, enabling both graceful
    /// shutdown from system signals and programmatic termination for testing
    /// scenarios.
    ///
    /// # Parameters
    ///
    /// * `signal` - A future that completes when the server should gracefully
    ///   shutdown. Common uses include system signal monitoring (SIGTERM,
    ///   SIGINT) for production deployments and programmatic triggers for
    ///   integration tests.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server starts and shuts down gracefully, or an
    /// [`Error`] if there are issues with server startup or operation.
    ///
    /// ## Parameters
    ///
    /// * `signal` - Future that resolves when the server should shutdown
    /// * `stream` - TCP listener stream for accepting incoming connections
    ///
    /// # Example
    ///
    /// ```
    /// use server::application::Application;
    /// use server::config::Config;
    /// use mock_server::MockServer;
    /// use tokio::signal;
    /// use tokio_stream::wrappers::TcpListenerStream;
    ///
    /// # async fn example(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    /// let mock = MockServer::new().await;
    /// let config = Config::test(pool, &mock).await?;
    /// let listener = tokio::net::TcpListener::bind(config.addr).await?;
    /// let stream = TcpListenerStream::new(listener);
    ///
    /// let app = Application::new(config)
    ///     .with_middlewares()
    ///     .with_services();
    ///
    /// // Run until Ctrl+C is pressed
    /// app.run(async { signal::ctrl_c().await.ok(); }, stream).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Type Constraints
    ///
    /// This method is only available on fully configured applications with
    /// the complete middleware stack (`Middleware<Identity>`) and PostgreSQL database connectivity.
    pub async fn run<F: Future<Output = ()>>(
        self,
        signal: F,
        stream: TcpListenerStream,
    ) -> Result<(), Error> {
        self.server.serve(stream, self.router.routes, signal).await
    }
}
