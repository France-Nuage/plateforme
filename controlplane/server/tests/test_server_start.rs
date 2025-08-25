use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_server_starts(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(pool.clone());
    let shutdown_tx = server::serve_with_tx(config).await?;
    assert!(shutdown_tx.send(()).is_ok());
    Ok(())
}
