use infrastructure::{
    Datacenter,
    v1::{ListDatacentersRequest, datacenters_client::DatacentersClient},
};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_datacenters_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let model = Datacenter::factory().create(&pool).await.unwrap();

    let config = Config::new(pool.clone());
    let addr = format!("http://{}", config.addr);
    let shutdown_tx = server::serve_with_tx(config).await?;
    let mut client = DatacentersClient::connect(addr).await?;

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
