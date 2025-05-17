use hypervisor_connector_proxmox::mock::{
    MockServer, WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock,
    WithVMCreateMock,
};
use hypervisors::Hypervisor;
use instances::v1::{
    CreateInstanceRequest, CreateInstanceResponse, instances_client::InstancesClient,
};
use server::{Server, ServerConfig};

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
    let hypervisor = Hypervisor {
        url: mock.url(),
        ..Default::default()
    };
    hypervisors::repository::create(&pool, &hypervisor)
        .await
        .unwrap();

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
            snippet: String::from("acme-snippet.yaml"),
        })
        .await;

    // Get the instance generated in the database
    let instance = &instances::repository::list(&pool).await.unwrap()[0];

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateInstanceResponse {
            instance: Some(instances::v1::Instance {
                id: instance.id.to_string(),
                created_at: Some(prost_types::Timestamp::default()),
                updated_at: Some(prost_types::Timestamp::default()),
                ..Default::default()
            })
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
