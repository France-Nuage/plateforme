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

/// Starts and runs the complete gRPC server application.
///
/// This function orchestrates the entire server startup process, including
/// signal handling setup, configuration loading, database connection
/// establishment, and application initialization. It runs the server until
/// a shutdown signal (SIGTERM or SIGINT) is received.
///
/// # Environment Variables
///
/// * `DATABASE_URL` - PostgreSQL connection string (required)
/// * `OIDC_URL` - OIDC provider discovery URL (optional, defaults to GitLab)
///
/// # Authentication Requirements
///
/// All gRPC service calls require a valid OIDC JWT token in the request metadata.
/// The server automatically validates tokens and rejects unauthenticated requests.
///
/// # Panics
///
/// This function will panic if:
/// - Signal handlers for SIGTERM or SIGINT cannot be installed
/// - PostgreSQL connection cannot be established
pub async fn serve() -> Result<(), crate::error::Error> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        shutdown_signal().await;
        // Send the shutdown signal. This will panic if it fails, as it prevents
        // to gracefully shutdown
        shutdown_tx
            .send(())
            .expect("could not send the shutdown signal");
    });

    let config = Config::from_env().await?;

    Application::new(config)
        .with_middlewares()
        .with_services()
        .run(async {
            shutdown_rx.await.ok();
        })
        .await
}

pub async fn serve_with_tx(config: Config) -> Result<oneshot::Sender<()>, crate::error::Error> {
    // Create a one-shot channel for sending a shutdown signal
    let (sender, receiver) = oneshot::channel();

    // Create and start the application in a separate task
    let app = Application::new(config)
        .with_middlewares()
        .with_services()
        .run(async {
            receiver.await.ok();
        });
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
async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {}
        _ = sigint.recv() => {}
    }
}
