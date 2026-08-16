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

use std::future::Future;

use tokio_stream::wrappers::TcpListenerStream;
use tonic_web::GrpcWebLayer;
use tower::layer::util::{Identity, Stack};
use tower_http::cors::CorsLayer;

use frn_core::billing::Billing;
use frn_core::billing::stripe::HttpStripeClient;
use frn_core::managed::ManagedServices;

use crate::config::Config;
use crate::error::Error;
use crate::router::Router;
use crate::server::{Server, TraceLayer};

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
type Middleware<L> = Stack<CorsLayer, Stack<TraceLayer, L>>;

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
            server: self.server.with_tracing().with_cors(
                self.config.allow_headers,
                self.config.allow_methods,
                self.config.allow_origin,
                self.config.expose_headers,
                self.config.allow_credentials,
            ),
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
        let instances = self.config.app.instances.clone();
        let invitations = self.config.app.invitations.clone();
        let organizations = self.config.app.organizations.clone();
        let projects = self.config.app.projects.clone();
        let users = self.config.app.users.clone();
        let zones = self.config.app.zones.clone();
        let auth = self.config.app.auth.clone();
        let worker_token = self.config.worker_token.clone();
        let ci_token = self.config.ci_token.clone();
        let managed_platform_config = self.config.managed_platform_config.clone();
        let kubeconfig_encryption_kek = self.config.kubeconfig_encryption_kek.clone();

        let mut router = self
            .router
            .health()
            .hypervisors(iam.clone(), pool.clone(), hypervisors.clone())
            .instances(iam.clone(), pool.clone(), instances.clone())
            .invitations(iam.clone(), invitations.clone(), users.clone())
            .profile(iam.clone())
            .managed_services(
                iam.clone(),
                pool.clone(),
                auth.clone(),
                ci_token,
                managed_platform_config.clone(),
                kubeconfig_encryption_kek.clone(),
            )
            .kubernetes_clusters(iam.clone(), pool.clone(), kubeconfig_encryption_kek.clone())
            .reflection()
            .resources(iam.clone(), organizations, pool.clone(), projects.clone())
            .zero_trust_networks(pool.clone())
            .zero_trust_network_types(pool.clone())
            .workflow_engine(pool.clone(), worker_token)
            .zones(iam.clone(), zones.clone());

        if let (Some(stripe_key), Some(webhook_secret), Some(success_url), Some(cancel_url)) = (
            self.config.stripe_secret_key.clone(),
            self.config.stripe_webhook_secret.clone(),
            self.config.stripe_checkout_success_url.clone(),
            self.config.stripe_checkout_cancel_url.clone(),
        ) {
            let stripe_client = HttpStripeClient::new(stripe_key);
            let managed_svc = ManagedServices::new(auth, pool.clone(), managed_platform_config);
            let billing = Billing::new(
                pool.clone(),
                stripe_client,
                managed_svc,
                kubeconfig_encryption_kek.clone(),
                success_url,
                cancel_url,
            );

            router = router.billing(iam.clone(), pool.clone(), billing, webhook_secret);
            tracing::info!("billing service enabled with Stripe integration");
        } else {
            let has_any = self.config.stripe_secret_key.is_some()
                || self.config.stripe_webhook_secret.is_some()
                || self.config.stripe_checkout_success_url.is_some()
                || self.config.stripe_checkout_cancel_url.is_some();
            if has_any {
                tracing::warn!(
                    stripe_secret_key = self.config.stripe_secret_key.is_some(),
                    stripe_webhook_secret = self.config.stripe_webhook_secret.is_some(),
                    stripe_checkout_success_url = self.config.stripe_checkout_success_url.is_some(),
                    stripe_checkout_cancel_url = self.config.stripe_checkout_cancel_url.is_some(),
                    "billing service disabled: partial Stripe configuration detected, all four variables are required"
                );
            } else {
                tracing::info!("billing service disabled (Stripe env vars not configured)");
            }
        }

        Self {
            config: self.config,
            router,
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
        if let Some(http_routes) = self.router.http_routes {
            let http_listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
                .await
                .map_err(Error::IO)?;
            tracing::info!("webhook HTTP server listening on 0.0.0.0:8081");
            tokio::spawn(async move {
                if let Err(e) = axum::serve(http_listener, http_routes).await {
                    tracing::error!(error = %e, "webhook HTTP server exited with error");
                }
            });
        }

        // Install the Prometheus recorder once and expose it at `/metrics`, so
        // the auth counters emitted by the BFF (and any future instrumentation)
        // are scrapable. Idempotent across the many server instances a test
        // process spins up.
        let _ = crate::metrics::handle();

        // gRPC-web is applied ONLY to the gRPC routes. tonic-web's layer returns
        // HTTP 400 for every non-gRPC request (`RequestKind::Other` → BAD_REQUEST
        // for HTTP/1.1, which is what a reverse proxy speaks to the backend), so
        // wrapping the whole service would make the plain-HTTP surfaces
        // (`/metrics`, BFF `/auth/*`) unreachable. axum's `.layer()` only wraps
        // the routes registered before the call, so the gRPC routes are wrapped
        // here and the HTTP routes below stay un-wrapped.
        let mut axum_router = self
            .router
            .routes
            .into_axum_router()
            .layer(GrpcWebLayer::new())
            .route("/metrics", axum::routing::get(metrics_endpoint));

        // Mount the confidential-client BFF (`/auth/*`) on the same origin as
        // gRPC-web, so the browser reaches it at the control-plane URL. Present
        // only when `OIDC_CLIENT_SECRET` is configured; when absent the BFF is not
        // mounted and the console cannot authenticate at all (the SPA/PKCE frontend
        // has been removed) — a deployment misconfiguration to avoid.
        if let Some(bff) = self.config.bff.clone() {
            tracing::info!(
                "BFF confidential-client auth enabled (/auth/login, /auth/callback, /auth/me, /auth/logout)"
            );
            axum_router = axum_router.merge(bff.into_router());
        }

        let svc = axum_router.into_service();
        self.server.serve(stream, svc, signal).await
    }
}

/// `GET /metrics` — renders the Prometheus text exposition format.
async fn metrics_endpoint() -> ([(axum::http::HeaderName, &'static str); 1], String) {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        crate::metrics::render(),
    )
}
