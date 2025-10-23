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
    // Setup tracing
    tracing_subscriber::fmt::init();

    // create the application configuration
    let config = Config::from_env().await?;

    let root_organization = config
        .app
        .organizations
        .clone()
        .initialize_root_organization(config.app.config.root_organization.name.clone())
        .await?;

    let root_service_account = config
        .app
        .service_accounts
        .clone()
        .initialize_root_service_account(
            &root_organization,
            config
                .app
                .config
                .root_organization
                .service_account_name
                .clone(),
            config
                .app
                .config
                .root_organization
                .service_account_key
                .clone(),
        )
        .await?;

    config
        .app
        .organizations
        .clone()
        .add_service_account(&root_organization, &root_service_account)
        .await?;

    // serve the application
    let sender = serve(config).await?;

    shutdown_signal().await;

    sender
        .send(())
        .expect("could not send shutdown signal to the application");

    Ok(())
}
