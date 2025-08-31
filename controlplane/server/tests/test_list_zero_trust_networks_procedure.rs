use infrastructure::{
    ZeroTrustNetwork,
    v1::{ListZeroTrustNetworksRequest, zero_trust_networks_client::ZeroTrustNetworksClient},
};
use mock_server::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_zero_trust_networks_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await;

    let model = ZeroTrustNetwork::factory()
        .for_default_organization()
        .for_default_zero_trust_network_type()
        .create(&pool)
        .await
        .unwrap();

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = ZeroTrustNetworksClient::connect(server_url).await?;

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
