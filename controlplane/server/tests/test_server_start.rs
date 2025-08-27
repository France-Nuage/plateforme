use auth::JwkValidator;
use hypervisor_connector_proxmox::mock::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_server_starts(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mock = MockServer::new().await;
    let oidc_url = mock.url();

    let config = Config::new(
        pool.clone(),
        JwkValidator::from_oidc_discovery(&oidc_url).await?,
    );
    let shutdown_tx = server::serve_with_tx(config).await?;
    assert!(shutdown_tx.send(()).is_ok());
    Ok(())
}
