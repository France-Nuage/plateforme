use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use server::{Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("establishing database connection...");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = Database::connect(&database_url).await?;
    Migrator::up(&connection, None).await?;
    println!("migrations executed");

    let config = ServerConfig {
        addr: Some(
            std::env::var("CONTROLPLANE_ADDR").unwrap_or_else(|_| (String::from("[::1]:80"))),
        ),
        api_url: std::env::var("PROXMOX_API_URL").expect("PROXMOX_API_URL not set"),
        authentication_header: std::env::var("PROXMOX_AUTHORIZATION_HEADER").ok(),
        console_url: std::env::var("CONSOLE_URL").ok(),
        connection,
    };

    Server::new(config).await?.serve().await?;
    Ok(())
}
