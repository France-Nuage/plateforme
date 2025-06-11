use hypervisors::{
    Hypervisor,
    v1::{DetachHypervisorRequest, hypervisors_client::HypervisorsClient},
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_detach_hypervisor_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let hypervisor = Hypervisor::factory()
        .for_organization_with(|organization| organization)
        .create(pool.clone())
        .await?;

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorsClient::connect(format!("http://{}", addr)).await?;

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
