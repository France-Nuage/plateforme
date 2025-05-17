use server::{Server, ServerConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a database connection and apply pending connections
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    sqlx::migrate!("../migrations").run(&pool).await?;

    // Create a configuration for the grpc server
    let config = ServerConfig {
        addr: Some(
            std::env::var("CONTROLPLANE_ADDR").unwrap_or_else(|_| (String::from("[::1]:80"))),
        ),
        console_url: std::env::var("CONSOLE_URL").ok(),
        pool,
    };

    info!("gonna start server...");

    // Create and server the grpc server
    Server::new(config).await?.serve().await
}
