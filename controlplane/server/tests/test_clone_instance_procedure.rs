use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock, WithVMCloneMock,
};
use instances::{
    Instance,
    v1::{CloneInstanceRequest, instances_client::InstancesClient},
};
use resources::organizations::Organization;
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_clone_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_next_id()
        .with_cluster_resource_list()
        .with_vm_clone()
        .with_task_status_read();
    let mock_url = mock.url();

    let organization = Organization::factory().create(&pool).await?;

    let instance = Instance::factory()
        .distant_id("100".into())
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .url(mock_url)
                .for_default_datacenter()
                .organization_id(organization.id)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .create(&pool)
        .await?;

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .clone_instance(CloneInstanceRequest {
            id: instance.id.to_string(),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
