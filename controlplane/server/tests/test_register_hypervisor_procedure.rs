use auth::mock::WithWellKnown;
use frn_core::{compute::Zone, resourcemanager::Organization};
use frn_rpc::v1::compute::{RegisterHypervisorRequest, hypervisors_client::HypervisorsClient};
use mock_server::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_register_hypervisor_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mock = MockServer::new().await.with_well_known();

    let zone = Zone::factory().create(&pool).await?;
    let organization = Organization::factory().create(&pool).await?;

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = HypervisorsClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .register(RegisterHypervisorRequest {
            zone_id: zone.id.to_string(),
            organization_id: organization.id.to_string(),
            ..Default::default()
        })
        .await;

    // Assert the result
    assert!(result.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
