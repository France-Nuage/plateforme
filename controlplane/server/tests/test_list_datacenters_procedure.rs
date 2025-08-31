use auth::mock::WithWellKnown;
use infrastructure::{
    Datacenter,
    v1::{ListDatacentersRequest, datacenters_client::DatacentersClient},
};
use mock_server::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_datacenters_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await.with_well_known();

    let model = Datacenter::factory().create(&pool).await.unwrap();

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = DatacentersClient::connect(server_url).await?;

    // Act the request to the rpc
    let response = client.list(ListDatacentersRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner().datacenters,
        vec![model.into()]
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
