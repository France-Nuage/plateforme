use auth::JwkValidator;
use hypervisor_connector_proxmox::mock::MockServer;
use infrastructure::{
    ZeroTrustNetworkType,
    v1::{
        ListZeroTrustNetworkTypesRequest,
        zero_trust_network_types_client::ZeroTrustNetworkTypesClient,
    },
};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_zero_trust_network_types_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await;
    let oidc_url = mock.url();

    let config = Config::new(
        pool.clone(),
        JwkValidator::from_oidc_discovery(&oidc_url).await?,
    );
    let model = ZeroTrustNetworkType::factory().create(&pool).await.unwrap();

    let addr = format!("http://{}", config.addr);
    let shutdown_tx = server::serve_with_tx(config).await?;
    let mut client = ZeroTrustNetworkTypesClient::connect(addr).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .list(ListZeroTrustNetworkTypesRequest::default())
        .await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner().zero_trust_network_types,
        vec![model.into()]
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
