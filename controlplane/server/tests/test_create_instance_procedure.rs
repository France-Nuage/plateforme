use database::Persistable;
use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock,
    WithVMCreateMock,
};
use hypervisors::Hypervisor;
use instances::v1::{
    CreateInstanceRequest, CreateInstanceResponse, instances_client::InstancesClient,
};
use resources::{DEFAULT_PROJECT_NAME, organizations::Organization, projects::Project};
use server::{Server, ServerConfig};
use sqlx::types::Uuid;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_next_id()
        .with_cluster_resource_list()
        .with_task_status_read()
        .with_vm_create();
    let mock_url = mock.url();

    let organization = Organization::factory().create(&pool).await?;

    Hypervisor::factory()
        .url(mock_url)
        .for_default_datacenter()
        .organization_id(organization.id)
        .create(&pool)
        .await?;

    let project = Project::factory()
        .name(DEFAULT_PROJECT_NAME.into())
        .organization_id(organization.id)
        .create(&pool)
        .await?;

    let config = ServerConfig::new(pool.clone());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .create_instance(CreateInstanceRequest {
            image: String::from("debian.qcow2"),
            cpu_cores: 1,
            memory_bytes: 536870912,
            name: String::from("acme-mgs"),
            project_id: project.id.to_string(),
            snippet: String::from("acme-snippet.yaml"),
        })
        .await;

    // Assert the result
    assert!(response.is_ok());
    let instance = &instances::Instance::list(&pool).await.unwrap()[0];
    assert_eq!(
        response.unwrap().into_inner(),
        CreateInstanceResponse {
            instance: Some(instances::v1::Instance {
                id: instance.id.to_string(),
                hypervisor_id: Uuid::default().to_string(),
                project_id: Uuid::default().to_string(),
                created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    instance.created_at
                ))),
                updated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    instance.updated_at
                ))),
                ..Default::default()
            })
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
