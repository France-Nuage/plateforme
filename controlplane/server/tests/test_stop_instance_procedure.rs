use hypervisor_connector_proxmox::mock::{
    WithClusterResourceList, WithTaskStatusReadMock, WithVMStatusStopMock,
};
use instances::{
    Instance,
    v1::{StopInstanceRequest, instances_client::InstancesClient},
};
use mock_server::MockServer;
use resources::organizations::Organization;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_stop_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_resource_list()
        .with_task_status_read()
        .with_vm_status_stop();
    let mock_url = mock.url();

    let organization = Organization::factory().create(&pool).await?;
    let instance = Instance::factory()
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .for_default_datacenter()
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .distant_id("100".into())
        .create(&pool)
        .await?;

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = InstancesClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .stop_instance(StopInstanceRequest {
            id: instance.id.to_string(),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
