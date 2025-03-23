use controlplane::server::{Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        addr: Some(String::from("[::1]:80")),
        api_url: std::env::var("PROXMOX_API_URL").expect("PROXMOX_API_URL not set"),
        authentication_header: std::env::var("PROXMOX_AUTHORIZATION_HEADER").ok(),
    };
    Server::new(config).await?.serve().await?;
    Ok(())
}
