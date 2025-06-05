use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterResourceList, WithTaskStatusReadMock, WithVMDeleteMock,
};
use instances::{
    Instance,
    v1::{DeleteInstanceRequest, instances_client::InstancesClient},
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_delete_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_resource_list()
        .with_task_status_read()
        .with_vm_delete();
    let mock_url = mock.url();

    let instance = Instance::factory()
        .distant_id("100".into())
        .for_hypervisor_with(|hypervisor| hypervisor.url(mock_url))
        .for_project_with(|project| project.for_organization_with(|organization| organization))
        .create(pool.clone())
        .await?;

    let config = ServerConfig::new(pool.clone());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .delete_instance(DeleteInstanceRequest {
            id: instance.id.to_string(),
        })
        .await;

    println!("instance: {:#?}", &instance);

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
