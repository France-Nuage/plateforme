use controlplane::server::{Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new(String::from("https://pve-poc01-internal.france-nuage.fr"));
    Server::new(config).await?.serve().await?;
    Ok(())
}
