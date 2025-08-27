use auth::JwkValidator;
use hypervisor_connector_proxmox::mock::MockServer;
use hypervisors::{
    Hypervisor,
    v1::{DetachHypervisorRequest, hypervisors_client::HypervisorsClient},
};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_detach_hypervisor_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mock = MockServer::new().await;
    let oidc_url = mock.url();

    let config = Config::new(
        pool.clone(),
        JwkValidator::from_oidc_discovery(&oidc_url).await?,
    );
    // Arrange the grpc server and a client
    let hypervisor = Hypervisor::factory()
        .for_default_datacenter()
        .for_organization_with(|organization| organization)
        .create(&pool)
        .await?;

    let addr = format!("http://{}", config.addr);
    let shutdown_tx = server::serve_with_tx(config).await?;
    let mut client = HypervisorsClient::connect(addr).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .detach_hypervisor(DetachHypervisorRequest {
            id: hypervisor.id.to_string(),
        })
        .await;

    // Assert the result
    assert!(result.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
