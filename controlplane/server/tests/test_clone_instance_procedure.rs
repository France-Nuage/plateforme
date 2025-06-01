use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock, WithVMCloneMock,
};
use hypervisors::Hypervisor;
use instances::{
    Instance,
    v1::{CloneInstanceRequest, instances_client::InstancesClient},
};
use resources::{organizations::Organization, projects::Project};
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

    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    let organization =
        resources::organizations::repository::create(&pool, &Organization::default())
            .await
            .expect("could not create organization");
    let project = resources::projects::repository::create(
        &pool,
        &Project {
            organization_id: organization.id,
            ..Default::default()
        },
    )
    .await
    .expect("could not create project");
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();
    let instance = Instance {
        hypervisor_id: hypervisor.id,
        project_id: project.id,
        distant_id: String::from("100"),
        ..Default::default()
    };
    instances::repository::create(&pool, &instance)
        .await
        .unwrap();

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
