//! HTTP/gRPC server implementation with middleware support.
//!
//! This module provides the [`Server`] structure that wraps tonic's transport
//! server with additional functionality for middleware composition, CORS
//! support, and distributed tracing. It offers a higher-level abstraction
//! over the underlying transport layer while maintaining full compatibility
//! with tonic's service ecosystem.

use auth::{AuthenticationLayer, Authz, OpenID};
use bytes::Bytes;
use http::{Request, Response};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::body::Body;
use tonic_web::GrpcWebLayer;
use tower::layer::util::{Identity, Stack};
use tower::{BoxError, Layer, Service};
use tower_http::classify::{GrpcErrorsAsFailures, SharedClassifier};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};

use crate::error::Error;

/// Type alias for [`tower_http::trace::TraceLayer`] configured with sane defaults.
///
/// This pre-configured trace layer uses [`GrpcErrorsAsFailures`] classifier to properly
/// categorize gRPC responses as successes or failures for tracing purposes.
pub type TraceLayer = tower_http::trace::TraceLayer<SharedClassifier<GrpcErrorsAsFailures>>;

/// A higher-level abstraction over [`tonic::transport::Server`] with batteries included.
///
/// This server wrapper provides a builder pattern similar to tonic's transport server,
/// but includes opinionated defaults for common middleware like tracing and CORS.
/// It encapsulates the complexity of setting up production-ready gRPC servers while
/// maintaining the flexibility of the underlying [`tonic::transport::Server`].
///
/// # Example
///
/// ```
/// # use server::server::Server;
/// let server = Server::new()
///     .with_tracing()
///     .with_cors(
///         tower_http::cors::AllowHeaders::any(),
///         tower_http::cors::AllowMethods::any(),
///         tower_http::cors::AllowOrigin::any(),
///         tower_http::cors::ExposeHeaders::any()
///         );
/// ```
///
/// [`tonic::transport::Server`]: https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html
pub struct Server<L = Identity> {
    /// The underlying [`tonic::transport::Server`] instance.
    ///
    /// [`tonic::transport::Server`]: https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html
    pub inner: tonic::transport::Server<L>,
}

impl Default for Server<Identity> {
    fn default() -> Self {
        Self::new()
    }
}

impl Server<Identity> {
    /// Creates a new [`Server`] instance with default configuration.
    ///
    /// This constructor initializes a new server with sensible defaults:
    /// - **HTTP/1.1 Support**: Enables HTTP/1.1 compatibility alongside HTTP/2
    /// - **Identity Layer**: Starts with no middleware layers applied
    ///
    /// The server can then be configured with additional middleware layers
    /// using the builder pattern methods.
    ///
    /// # Example
    ///
    /// ```
    /// # use server::server::Server;
    /// let server = Server::new();
    /// ```
    pub fn new() -> Self {
        Server {
            inner: tonic::transport::Server::default().accept_http1(true),
        }
    }
}

impl<L> Server<L> {
    /// Starts the gRPC server with graceful shutdown support.
    ///
    /// This method starts the underlying tonic transport server using an
    /// incoming connection stream and serving the provided service until the
    /// shutdown signal completes. It enables graceful shutdown handling,
    /// allowing in-flight requests to complete before terminating.
    ///
    /// # Parameters
    ///
    /// * `stream` - TCP listener stream for accepting incoming connections
    /// * `svc` - The service implementation to serve
    /// * `signal` - A future that triggers graceful shutdown when it completes
    ///
    /// # Type Constraints
    ///
    /// This method has complex generic constraints to work with tonic's
    /// layered service architecture. The implementation is adapted from
    /// tonic's transport server.
    ///
    /// # Reference
    ///
    /// Implementation adapted from:
    /// https://github.com/hyperium/tonic/blob/master/tonic/src/transport/server/mod.rs#L588
    pub async fn serve<S, F, ResBody>(
        self,
        stream: TcpListenerStream,
        svc: S,
        signal: F,
    ) -> Result<(), Error>
    where
        L: Layer<S>,
        L::Service: Service<Request<Body>, Response = Response<ResBody>> + Clone + Send + 'static,
        <<L as Layer<S>>::Service as Service<Request<Body>>>::Future: Send,
        <<L as Layer<S>>::Service as Service<Request<Body>>>::Error:
            Into<BoxError> + Send + 'static,
        F: Future<Output = ()>,
        ResBody: http_body::Body<Data = Bytes> + Send + 'static,
        ResBody::Error: Into<BoxError>,
    {
        self.inner
            .serve_with_incoming_shutdown(svc, stream, signal)
            .await
            .map_err(Into::into)
    }

    /// Adds JWT authentication middleware to the server.
    ///
    /// This method applies an [`AuthenticationLayer`] that validates JWT tokens
    /// on all incoming requests and injects IAM context for downstream services.
    /// The authentication middleware uses OIDC provider configuration to validate
    /// JWT signatures and extract claims.
    ///
    /// # Parameters
    ///
    /// * `openid` - OpenID provider configured with OIDC information
    ///
    /// # Example
    ///
    /// ```
    /// # use server::server::Server;
    /// # use auth::{Authz, OpenID};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mock = mock_server::MockServer::new().await;
    /// let openid = OpenID::discover(reqwest::Client::new(), &format!("{}/.well-known/openid-configuration", &mock.url())).await?;
    /// let authz = Authz::mock().await;
    /// let server = Server::new()
    ///     .with_authentication(authz, openid);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This is equivalent to calling [`tonic::transport::Server::layer`] with
    /// an [`AuthenticationLayer`] configured with the provided OpenID provider.
    ///
    /// [`tonic::transport::Server::layer`]: https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html#method.layer
    pub fn with_authentication(
        self,
        authz: Authz,
        openid: OpenID,
    ) -> Server<Stack<AuthenticationLayer, L>> {
        Server {
            inner: self.inner.layer(AuthenticationLayer::new(authz, openid)),
        }
    }

    /// Add distributed tracing support to the server.
    ///
    /// This applies a pre-configured [`tower_http::trace::TraceLayer`] that provides
    /// structured logging and distributed tracing for gRPC services. The trace layer
    /// uses [`GrpcErrorsAsFailures`] classifier to properly categorize gRPC responses
    /// for observability tools.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use server::server::Server;
    ///
    /// let server = Server::new()
    ///     .with_tracing();
    /// ```
    ///
    /// This is equivalent to calling [`tonic::transport::Server::layer`] with
    /// [`tower_http::trace::TraceLayer::new_for_grpc()`], but with additional
    /// configuration for gRPC-specific concerns.
    ///
    /// [`tower_http::trace::TraceLayer`]: https://docs.rs/tower-http/latest/tower_http/trace/struct.TraceLayer.html
    /// [`tower_http::trace::TraceLayer::new_for_grpc()`]: https://docs.rs/tower-http/latest/tower_http/trace/struct.TraceLayer.html#method.new_for_grpc
    /// [`tonic::transport::Server::layer`]: https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html#method.layer
    /// [`GrpcErrorsAsFailures`]: https://docs.rs/tower-http/latest/tower_http/classify/struct.GrpcErrorsAsFailures.html
    pub fn with_tracing(self) -> Server<Stack<TraceLayer, L>> {
        Server {
            inner: self.inner.layer(TraceLayer::new_for_grpc()),
        }
    }

    /// Adds gRPC-Web support to the server.
    ///
    /// This method applies a [`GrpcWebLayer`] that enables gRPC-Web protocol support,
    /// allowing browser-based clients to connect to the gRPC server. This is essential
    /// for web applications that need to make gRPC calls directly from JavaScript.
    ///
    /// # Example
    ///
    /// ```
    /// # use server::server::Server;
    /// let server = Server::new()
    ///     .with_web();
    /// ```
    ///
    /// This is equivalent to calling [`tonic::transport::Server::layer`] with
    /// a [`GrpcWebLayer`] configured with default settings.
    ///
    /// [`tonic::transport::Server::layer`]: https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html#method.layer
    /// [`GrpcWebLayer`]: https://docs.rs/tonic-web/latest/tonic_web/struct.GrpcWebLayer.html
    pub fn with_web(self) -> Server<Stack<GrpcWebLayer, L>> {
        Server {
            inner: self.inner.layer(GrpcWebLayer::new()),
        }
    }

    /// Add Cross-Origin Resource Sharing (CORS) support to the server.
    ///
    /// This applies a [`tower_http::cors::CorsLayer`] configured with the specified
    /// origin and method restrictions. The CORS layer enables cross-origin requests
    /// based on the provided [`AllowOrigin`] and [`AllowMethods`] configurations,
    /// making it flexible for both development and production environments.
    ///
    /// # Parameters
    ///
    /// * `allow_origin` - Specifies which origins are permitted to make cross-origin requests
    /// * `allow_methods` - Specifies which HTTP methods are allowed for cross-origin requests
    ///
    /// # Example
    ///
    /// ```
    /// # use server::server::Server;
    /// # use tower_http::cors::{AllowOrigin, AllowMethods};
    /// let server = Server::new().with_cors(
    ///     tower_http::cors::AllowHeaders::any(),
    ///     tower_http::cors::AllowMethods::any(),
    ///     tower_http::cors::AllowOrigin::any(),
    ///     tower_http::cors::ExposeHeaders::any()
    /// );
    /// ```
    ///
    /// This is equivalent to calling [`tonic::transport::Server::layer`] with a
    /// [`tower_http::cors::CorsLayer`] configured with the specified parameters.
    ///
    /// # Note
    ///
    /// The flexibility of this method allows for both permissive configurations suitable
    /// for development (using [`AllowOrigin::any()`] and [`AllowMethods::any()`]) and
    /// restrictive configurations appropriate for production deployments.
    ///
    /// [`tower_http::cors::CorsLayer`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.CorsLayer.html
    /// [`tonic::transport::Server::layer`]: https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html#method.layer
    pub fn with_cors(
        self,
        allow_headers: AllowHeaders,
        allow_methods: AllowMethods,
        allow_origin: AllowOrigin,
        expose_headers: ExposeHeaders,
    ) -> Server<Stack<CorsLayer, L>> {
        Server {
            inner: self.inner.layer(
                CorsLayer::new()
                    .allow_headers(allow_headers)
                    .allow_methods(allow_methods)
                    .allow_origin(allow_origin)
                    .expose_headers(expose_headers),
            ),
        }
    }
}
