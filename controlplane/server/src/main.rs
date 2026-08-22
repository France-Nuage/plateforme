//! Main executable for the control plane.
//!
//! Runs the gRPC server by default, or a `catalog` subcommand that reconciles
//! or checks the declarative catalogue and exits. Command parsing uses clap.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use server::catalog::{self, DEFAULT_CATALOG_PATH};
use server::{Config, serve, shutdown_signal};

/// Control plane binary: gRPC server, plus catalogue management subcommands.
#[derive(Parser)]
#[command(name = "server", about = "France Nuage control plane")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Manage the declarative service catalogue.
    #[command(subcommand)]
    Catalog(CatalogCommand),
}

#[derive(Subcommand)]
enum CatalogCommand {
    /// Parse and validate the catalogue only (no Stripe, no database).
    Validate {
        /// Path to catalog.yaml (defaults to the bundled catalogue).
        #[arg(default_value = DEFAULT_CATALOG_PATH)]
        path: PathBuf,
    },
    /// Reconcile the catalogue into Stripe and the database.
    Sync {
        #[arg(default_value = DEFAULT_CATALOG_PATH)]
        path: PathBuf,
    },
    /// Archive catalogue-owned Stripe objects no longer declared.
    Prune {
        #[arg(default_value = DEFAULT_CATALOG_PATH)]
        path: PathBuf,
        /// Apply the changes; without this flag the run is a dry run.
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), server::error::Error> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Some(Command::Catalog(command)) = cli.command {
        return run_catalog_command(command).await;
    }

    run_server().await
}

/// Runs a one-shot `catalog` subcommand and exits.
async fn run_catalog_command(command: CatalogCommand) -> Result<(), server::error::Error> {
    match command {
        // Pure parse/validate: no Config, no Stripe, no database.
        CatalogCommand::Validate { path } => catalog::run_validate(&path),
        CatalogCommand::Sync { path } => {
            let config = Config::from_env().await?;
            catalog::run_sync(config, &path).await
        }
        CatalogCommand::Prune { path, force } => {
            let config = Config::from_env().await?;
            catalog::run_prune(config, &path, force).await
        }
    }
}

/// Boots the control plane: self-initializes its baseline state, then serves.
async fn run_server() -> Result<(), server::error::Error> {
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

    // Without a seeded admin, a fresh install can register no hosting cluster
    // nor perform any platform-admin action.
    if let Some(admin_email) = config.app.config.root_organization.admin_email.clone() {
        config
            .app
            .users
            .clone()
            .initialize_root_admin(admin_email)
            .await?;
    }

    catalog::sync_at_boot(&config).await?;
    catalog::spawn_version_discovery(&config);

    let sender = serve(config).await?;

    shutdown_signal().await;

    sender
        .send(())
        .expect("could not send shutdown signal to the application");

    Ok(())
}
