use super::instance_service::InstanceService;
use proto::instance_server::InstanceServer;
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server as TonicServer;
use tonic::transport::server::Router;

/// Provide a gRPC tonic server.
///
/// The Server struct is a wrapper around the tonic gRPC server, which allows centralizing the
/// server configuration here in the server crate, rather than have it defined in the main binary
/// crate.
pub struct Server {
    pub addr: SocketAddr,
    pub router: Router,
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
                reqwest::header::HeaderValue::from_str(&authentication_header)?,
            );
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        // Create the tonic router
        let mut server = TonicServer::builder();
        let router = server.add_service(InstanceServer::new(InstanceService::new(
            config.api_url.clone(),
            client.clone(),
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
}

impl ServerConfig {
    pub fn new(api_url: String) -> Self {
        ServerConfig {
            addr: None,
            api_url,
            authentication_header: None,
        }
    }
}
