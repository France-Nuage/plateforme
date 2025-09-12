//! gRPC server library for the control plane.
//!
//! This library provides a complete gRPC server implementation with batteries
//! included for typical microservice scenarios. It orchestrates application
//! components including configuration management, PostgreSQL database connectivity,
//! service routing, middleware composition, and graceful shutdown handling.
//!
//! The library is designed around a builder pattern that allows progressive
//! configuration of server components, making it suitable for both development
//! and production deployments.
//!
//! # Authentication
//!
//! This server requires authentication via OpenID Connect (OIDC) JWT tokens. All
//! gRPC requests must include a valid JWT token in the `authorization` metadata
//! field using the Bearer token scheme:
//!
//! ```text
//! authorization: Bearer <jwt_token>
//! ```
//!
//! The authentication middleware automatically validates JWT tokens using OIDC
//! discovery and JWK (JSON Web Key) validation. Requests without valid tokens
//! will be rejected with an authentication error.

pub mod application;
pub mod config;
pub mod error;
pub mod router;
pub mod server;
pub use application::Application;
pub use config::Config;
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::oneshot,
};
use tokio_stream::wrappers::TcpListenerStream;

/// Starts a gRPC server with the provided configuration and returns a shutdown handle.
///
/// This function initializes and starts a complete gRPC server application with
/// authentication middleware, service routing, and graceful shutdown capabilities.
/// The server runs in a background task and can be controlled via the returned
/// shutdown sender.
///
/// ## Parameters
///
/// * `config` - Server configuration including network settings, CORS policies,
///   database pool, and JWT validation settings
///
/// ## Returns
///
/// Returns a `oneshot::Sender<()>` that can be used to trigger graceful shutdown
/// of the server. When a signal is sent through this channel, the server will
/// stop accepting new connections and gracefully shutdown.
///
/// ## Architecture
///
/// The function performs these steps:
/// 1. Creates a shutdown signal channel for coordination
/// 2. Binds a TCP listener to the configured address
/// 3. Builds the application with middleware stack and services
/// 4. Spawns the server in a background task
/// 5. Returns the shutdown trigger
///
/// ## Usage
///
/// ```
/// # use server::{Config, serve};
/// # use mock_server::MockServer;
/// # async fn example(pool: &sqlx::PgPool) -> Result<(), server::error::Error> {
/// let mock = MockServer::new().await;
/// let config = Config::test(pool, &mock).await?;
/// let shutdown_tx = serve(config).await?;
///
/// // Server is now running in background
/// // Send shutdown signal when ready
/// shutdown_tx.send(()).expect("server should be running");
/// # Ok(())
/// # }
/// ```
///
/// ## Error Conditions
///
/// Returns an error if:
/// - TCP listener cannot bind to the configured address
/// - Application initialization fails
pub async fn serve(config: Config) -> Result<oneshot::Sender<()>, crate::error::Error> {
    tracing::info!("starting the application...");
    // Create a one-shot channel for sending a shutdown signal
    let (sender, receiver) = oneshot::channel();
    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    let stream = TcpListenerStream::new(listener);

    // Create and start the application in a separate task
    let app = Application::new(config)
        .with_middlewares()
        .with_services()
        .run(
            async {
                tracing::info!("waiting for shutdown signal...");
                receiver.await.ok();
                tracing::info!("signal received, shutting down...")
            },
            stream,
        );
    tokio::spawn(app);

    // Return the shutdown signal sender handle
    Ok(sender)
}

/// Waits for system shutdown signals (SIGTERM or SIGINT).
///
/// This function sets up signal handlers for graceful shutdown and blocks
/// until either a SIGTERM or SIGINT signal is received. It's used internally
/// by the [`serve`] function to coordinate graceful server shutdown.
///
/// # Panics
///
/// This function will panic if signal handlers for SIGTERM or SIGINT cannot
/// be installed on the current system.
pub async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {}
        _ = sigint.recv() => {}
    }
}
