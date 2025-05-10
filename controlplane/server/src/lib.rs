use hyper::http;
use hypervisors::{rpc::HypervisorsRpcService, v1::hypervisors_server::HypervisorsServer};
use instances::InstancesRpcService;
use instances::v1::instances_server::InstancesServer;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server as TonicServer;
use tower_http::cors::{Any, CorsLayer};

/// Provide a gRPC tonic server.
///
/// The Server struct is a wrapper around the tonic gRPC server, which allows centralizing the
/// server configuration here in the server crate, rather than have it defined in the main binary
/// crate.
pub struct Server {
    pub addr: SocketAddr,
    pub router: tonic::transport::server::Router<
        tower_layer::Stack<tower_http::cors::CorsLayer, tower_layer::Identity>,
    >,
}

impl Server {
    /// Create a new gRPC server for the controlplane.
    pub async fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Compute the socket address
        let addr: SocketAddr = match config.addr {
            Some(addr) => addr.parse()?,
            None => tokio::net::TcpListener::bind("[::1]:0")
                .await?
                .local_addr()?,
        };

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<HypervisorsServer<HypervisorsRpcService>>()
            .await;
        health_reporter
            .set_serving::<InstancesServer<InstancesRpcService>>()
            .await;

        let hypervisors_service =
            HypervisorsServer::new(HypervisorsRpcService::new(config.pool.clone()));
        let instances_service = InstancesServer::new(InstancesRpcService::new(config.pool.clone()));

        let cors = CorsLayer::new()
            .allow_origin(
                config
                    .console_url
                    .unwrap_or(String::from("http://localhost"))
                    .parse::<http::HeaderValue>()
                    .map_err(|e| format!("Invalid CORS origin header: {}", e))?,
            )
            .allow_methods(Any)
            .allow_headers(Any);
        // Create the tonic router
        let server = TonicServer::builder().accept_http1(true);
        let router = server
            .layer(cors)
            .add_service(health_service)
            .add_service(tonic_web::enable(hypervisors_service))
            .add_service(tonic_web::enable(instances_service));

        // Return a Server instance
        Ok(Server { addr, router })
    }

    /// Serve the gRPC server on the configured address.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        self.router.serve(self.addr).await?;
        Ok(())
    }

    /// Serve the gRPC server and accept a signal to gracefully shut it down.
    pub async fn serve_with_shutdown(
        self,
    ) -> Result<oneshot::Sender<()>, Box<dyn std::error::Error>> {
        // Create a TCP listener bound to the address
        let listener = tokio::net::TcpListener::bind(self.addr).await?;

        // Create the shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Spawn the server in a separate task
        tokio::spawn(async move {
            self.router
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    shutdown_rx.await.ok();
                })
                .await
        });

        Ok(shutdown_tx)
    }
}

/// Define the configuration options for the gRPC server.
#[derive(Debug)]
pub struct ServerConfig {
    pub addr: Option<String>,
    pub console_url: Option<String>,
    pub pool: sqlx::PgPool,
}

impl ServerConfig {
    pub fn new(pool: PgPool) -> Self {
        ServerConfig {
            addr: None,
            console_url: None,
            pool,
        }
    }
}
