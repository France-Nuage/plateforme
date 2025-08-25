use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterResourceList, WithTaskStatusReadMock, WithVMDeleteMock,
};
use instances::{
    Instance,
    v1::{DeleteInstanceRequest, instances_client::InstancesClient},
};
use resources::organizations::Organization;
use server::Config;

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

    let organization = Organization::factory().create(&pool).await?;

    let instance = Instance::factory()
        .distant_id("100".into())
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .for_default_datacenter()
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .create(&pool)
        .await?;

    let config = Config::new(pool.clone());
    let addr = format!("http://{}", config.addr);
    let shutdown_tx = server::serve_with_tx(config).await?;
    let mut client = InstancesClient::connect(addr).await?;

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
