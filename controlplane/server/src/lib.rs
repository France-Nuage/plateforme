use hyper::http;
use hypervisor::{rpc::HypervisorsRpcService, v1::hypervisors_server::HypervisorsServer};
use instance::InstancesRpcService;
use instance::v1::instances_server::InstancesServer;
use sea_orm::DatabaseConnection;
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

        // Create a reqwest client with authentication headers
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(authentication_header) = config.authentication_header {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "PVEAPIToken={}",
                    authentication_header
                ))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

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
            .add_service(tonic_web::enable(InstancesServer::new(
                InstancesRpcService::new(config.api_url.clone(), client.clone()),
            )))
            .add_service(tonic_web::enable(HypervisorsServer::new(
                HypervisorsRpcService::new(config.connection.clone()),
            )));

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
#[derive(Debug, Default)]
pub struct ServerConfig {
    pub addr: Option<String>,
    pub api_url: String,
    pub authentication_header: Option<String>,
    pub connection: DatabaseConnection,
    pub console_url: Option<String>,
}

impl ServerConfig {
    pub fn new(api_url: String, connection: DatabaseConnection) -> Self {
        ServerConfig {
            addr: None,
            api_url,
            authentication_header: None,
            connection,
            console_url: None,
        }
    }
}
