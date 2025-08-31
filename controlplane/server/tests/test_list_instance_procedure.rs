use hypervisor_connector_proxmox::mock::WithClusterResourceList;
use instances::{
    Instance,
    v1::{ListInstancesRequest, instances_client::InstancesClient},
};
use mock_server::MockServer;
use resources::{DEFAULT_PROJECT_NAME, organizations::Organization};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_instances_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_cluster_resource_list();
    let mock_url = mock.url();

    let organization = Organization::factory().create(&pool).await?;
    Instance::factory()
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .for_default_datacenter()
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project_with(move |project| {
            project
                .name(DEFAULT_PROJECT_NAME.into())
                .organization_id(organization.id)
        })
        .create(&pool)
        .await?;

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = InstancesClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client.list_instances(ListInstancesRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    let instances = response.unwrap().into_inner().instances;
    assert_eq!(instances.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
