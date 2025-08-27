use auth::JwkValidator;
use hypervisor_connector_proxmox::mock::MockServer;
use infrastructure::{
    ZeroTrustNetwork,
    v1::{ListZeroTrustNetworksRequest, zero_trust_networks_client::ZeroTrustNetworksClient},
};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_zero_trust_networks_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await;
    let oidc_url = mock.url();

    let config = Config::new(
        pool.clone(),
        JwkValidator::from_oidc_discovery(&oidc_url).await?,
    );
    let model = ZeroTrustNetwork::factory()
        .for_default_organization()
        .for_default_zero_trust_network_type()
        .create(&pool)
        .await
        .unwrap();

    let addr = format!("http://{}", config.addr);
    let shutdown_tx = server::serve_with_tx(config).await?;
    let mut client = ZeroTrustNetworksClient::connect(addr).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client.list(ListZeroTrustNetworksRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner().zero_trust_networks,
        vec![model.into()]
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
