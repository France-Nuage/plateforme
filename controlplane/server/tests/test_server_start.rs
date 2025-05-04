use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_server_starts(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(ServerConfig::new(pool)).await?;
    let shutdown_tx = server.serve_with_shutdown().await?;
    shutdown_tx.send(()).ok();
    Ok(())
}
