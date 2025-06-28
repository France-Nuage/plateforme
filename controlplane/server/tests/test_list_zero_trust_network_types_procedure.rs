use infrastructure::{
    ZeroTrustNetworkType,
    v1::{
        ListZeroTrustNetworkTypesRequest,
        zero_trust_network_types_client::ZeroTrustNetworkTypesClient,
    },
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_zero_trust_network_types_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let model = ZeroTrustNetworkType::factory().create(&pool).await.unwrap();

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = ZeroTrustNetworkTypesClient::connect(format!("http://{}", addr)).await?;

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
