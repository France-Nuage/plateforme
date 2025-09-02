//! Main executable for the gRPC server application.
//!
//! This binary serves as the entry point for the control plane gRPC server.
//! It initializes the tokio async runtime and delegates to the server library
//! for complete application orchestration.

use server::{Config, serve, shutdown_signal};

/// Entry point for the gRPC server application.
///
/// This function initializes the tokio async runtime and starts the complete
/// server application by calling the [`server::serve`] function. It serves
/// as the bridge between the synchronous main entry point and the async
/// server implementation.
#[tokio::main]
async fn main() -> Result<(), server::error::Error> {
    // create the application configuration
    let config = Config::from_env().await?;

    // serve the application
    tracing::info!("starting the application...");
    let sender = serve(config).await?;

    tracing::info!("waiting for shutdown signal...");
    shutdown_signal().await;
    tracing::info!("shutdown signal received, gracefully shutting down...");

    sender
        .send(())
        .expect("could not send shutdown signal to the application");

    Ok(())
}
