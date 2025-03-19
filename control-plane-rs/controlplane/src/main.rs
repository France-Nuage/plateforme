use controlplane::server::ServerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    controlplane::server::Server::new(ServerConfig::default())
        .await?
        .serve()
        .await?;
    Ok(())
}
