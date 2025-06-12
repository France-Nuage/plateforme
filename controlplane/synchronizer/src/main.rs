use std::{error::Error, sync::Arc, time::Duration};

use instances::InstancesService;
use synchronizer::{heartbeat, synchronize};
use tokio::{sync::Mutex, time};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    info!("Starting synchronizer service...");

    // Setup service
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    let instances_service = InstancesService::new(pool);

    // Setup ticker
    let tick = std::env::var("INTERVAL")
        .unwrap_or_else(|_| String::from("5"))
        .parse::<u64>()
        .unwrap_or(5);
    let sync_in_progress = Arc::new(Mutex::new(false));
    let mut interval = time::interval(Duration::from_secs(tick));

    // Setup heartbeat
    let client = reqwest::Client::new();
    let heartbeat_url = std::env::var("HEARTBEAT_URL").ok();

    loop {
        // Start the ticker
        interval.tick().await;

        // Check if sync is already running
        if *sync_in_progress.lock().await {
            warn!("Sync already in progress, skipping this tick");
            continue;
        }

        // Call the synchronization process
        match synchronize(&instances_service).await {
            // If the synchronization worked, trigger a heartbeat if the url is defined
            Ok(_) => heartbeat(&client, &heartbeat_url).await,
            // Otherwise log an error
            Err(e) => error!(error = %e, "Sync failed"),
        }

        // Mark sync as complete
        *sync_in_progress.lock().await = false;
    }
}
