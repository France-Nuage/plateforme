use server::{Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    sqlx::migrate!("../migrations").run(&pool).await?;

    let config = ServerConfig {
        addr: Some(
            std::env::var("CONTROLPLANE_ADDR").unwrap_or_else(|_| (String::from("[::1]:80"))),
        ),
        console_url: std::env::var("CONSOLE_URL").ok(),
        pool,
    };

    Server::new(config).await?.serve().await?;
    Ok(())
}
