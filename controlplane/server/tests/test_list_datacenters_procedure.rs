use auth::JwkValidator;
use hypervisor_connector_proxmox::mock::MockServer;
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
    let mock = MockServer::new().await;
    let oidc_url = mock.url();

    let config = Config::new(
        pool.clone(),
        JwkValidator::from_oidc_discovery(&oidc_url).await?,
    );
    let model = Datacenter::factory().create(&pool).await.unwrap();

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
