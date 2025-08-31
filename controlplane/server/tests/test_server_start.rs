use mock_server::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_server_starts(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mock = MockServer::new().await;

    let config = Config::test(&pool, &mock).await?;
    let shutdown_tx = server::serve(config).await?;
    assert!(shutdown_tx.send(()).is_ok());
    Ok(())
}
