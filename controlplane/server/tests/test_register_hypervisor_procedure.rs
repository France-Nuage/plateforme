use hypervisors::v1::{RegisterHypervisorRequest, hypervisors_client::HypervisorsClient};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_register_hypervisor_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = HypervisorsClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let result = client
        .register_hypervisor(RegisterHypervisorRequest::default())
        .await;

    // Assert the result
    assert!(result.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
