use tokio::time::interval;
use std::time::Duration;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Url;
use log::{debug, error, info, trace, warn};
use clap::{crate_name, crate_version, Parser};
use sqlx::{query, query_as, PgConnection, PgPool};
use anyhow::anyhow;

mod work;
use work::*;

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
struct Config {
    /// Database URL (with credentials)
    #[clap(long, env, hide_env_values = true)]
    database_url: String,

    /// Number of request attempts to handle concurrently
    #[clap(long, env, default_value = "1", value_parser=clap::value_parser!(u8).range(1..=100))]
    concurrent: u8,

    /// Worker name (as defined in the infrastructure.worker table)
    #[clap(long, env)]
    worker_name: String,

    /// Worker version (if empty, will use version from Cargo.toml)
    #[clap(long, env)]
    worker_version: Option<String>,

    /// Worker refresh interval
    #[clap(long, env, default_value = "5", value_parser=clap::value_parser!(u64).range(1..=100))]
    worker_refresh_interval: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    let mut ticker = interval(Duration::from_secs(config.worker_refresh_interval));

    let worker_name = config.worker_name.to_owned();
        let worker_version = config
            .worker_version
            .to_owned()
            .unwrap_or_else(|| crate_version!().to_owned());

    info!(
            "Starting {} {worker_version} [{worker_name}]",
            crate_name!(),
        );


    println!("{:?}", config);

    debug!("Connecting to database...");
    let connect_options = config
            .database_url
            .parse::<PgConnectOptions>()?
            .application_name(&format!("{}-{worker_version}-{worker_name}", crate_name!()));

        // Création du pool de connexions
        let pool = PgPoolOptions::new()
            .max_connections(config.concurrent.into())
            .connect_with(connect_options)
            .await?;
    info!("Connected to database");


    loop {
        ticker.tick().await;

        let p = pool.clone();

//         get_vm_instances(p).await.unwrap();
        println!("Getting VM instances...");
        tokio::spawn(async move {
            match get_vm_instances(p).await {
                Ok(records) => {
                    println!("Got {} instances", records.len());
                    // Vous pouvez ici traiter chaque instance sans bloquer la boucle
                    // p.ex: for record in records { ... }
                }
                Err(e) => eprintln!("Failed to get VM instances: {}", e),
            }
        });
    }

    Ok(())
}