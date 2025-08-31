//! Main executable for the gRPC server application.
//!
//! This binary serves as the entry point for the control plane gRPC server.
//! It initializes the tokio async runtime and delegates to the server library
//! for complete application orchestration.

use server::{Config, serve, shutdown_signal};
use tracing::info;

/// Entry point for the gRPC server application.
///
/// This function initializes the tokio async runtime and starts the complete
/// server application by calling the [`server::serve`] function. It serves
/// as the bridge between the synchronous main entry point and the async
/// server implementation.
#[tokio::main]
async fn main() -> Result<(), server::error::Error> {
    let config = Config::from_env().await?;

    let output = tokio::select! {
       _ = shutdown_signal() => "shutdown signal received, exiting gracefully",
        _ = serve(config) => "application completed successfully",
    };

    info!(output);

    Ok(())
}
