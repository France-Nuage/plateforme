use frn_core::authorization::Relationship;
use spicedb::SpiceDB;
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    tracing_subscriber::fmt().init();

    tracing::info!("starting worker...");

    // retrieve environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let spicedb_url = env::var("SPICEDB_URL").expect("SPICEDB_URL must be set");
    let spicedb_token =
        env::var("SPICEDB_GRPC_PRESHARED_KEY").expect("SPICEDB_GRPC_PRESHARED_KEY must be set");

    // instanciate the required services
    let pool = PgPool::connect(&database_url).await?;
    let mut auth = SpiceDB::connect(&spicedb_url, &spicedb_token).await?;

    // unwind pre-existing entries in the database
    while let Some(relationship) = Relationship::consume(pool.clone(), &mut auth).await? {
        tracing::info!("processed relationship {}", &relationship);
    }

    Relationship::subscribe(pool, &mut auth).await?;

    Ok(())
}
