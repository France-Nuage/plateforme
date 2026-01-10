use frn_core::longrunning::Operation;
use operation_worker::Worker;
use spicedb::SpiceDB;
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    tracing::info!("starting operation worker...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let spicedb_url = env::var("SPICEDB_URL").expect("SPICEDB_URL must be set");
    let spicedb_token =
        env::var("SPICEDB_GRPC_PRESHARED_KEY").expect("SPICEDB_GRPC_PRESHARED_KEY must be set");

    let pool = PgPool::connect(&database_url).await?;
    let authorizer = SpiceDB::connect(&spicedb_url, &spicedb_token).await?;

    let mut worker = Worker::new(authorizer, pool.clone());

    // Process pending operations
    while let Some(operation) = worker.consume().await? {
        worker.execute(&operation).await?;
        Operation::mark_completed(operation.id, &pool).await?;
        tracing::info!("processed operation {}", operation.id);
    }

    // Subscribe to new operations
    worker.subscribe().await?;

    Ok(())
}
