// use hypervisor_connector_proxmox::mock::{MockServer, WithClusterResourceList};
// use hypervisors::Hypervisor;
// use instances::v1::{ListInstancesRequest, instances_client::InstancesClient};
// use resources::{DEFAULT_PROJECT_NAME, organizations::Organization, projects::Project};
// use server::{Server, ServerConfig};
//
// #[sqlx::test(migrations = "../migrations")]
// async fn test_the_list_instances_procedure_works(
//     pool: sqlx::PgPool,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Arrange the grpc server and a client
//     let mock = MockServer::new().await.with_cluster_resource_list();
//     let hypervisor = Hypervisor {
//         url: mock.url(),
//         ..Default::default()
//     };
//     let hypervisor = hypervisors::repository::create(&pool, hypervisor)
//         .await
//         .unwrap();
//     let organization = resources::organizations::repository::create(&pool, Organization::default())
//         .await
//         .expect("could not create organization");
//     resources::projects::repository::create(
//         &pool,
//         Project {
//             organization_id: organization.id,
//             name: String::from(DEFAULT_PROJECT_NAME),
//             ..Default::default()
//         },
//     )
//     .await
//     .expect("could not create project");
//     let config = ServerConfig::new(pool);
//     let server = Server::new(config).await?;
//     let addr = server.addr;
//     let shutdown_tx = server.serve_with_shutdown().await?;
//     let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;
//
//     // Act the request to the test_the_status_procedure_works
//     let response = client.list_instances(ListInstancesRequest::default()).await;
//
//     // Assert the result
//     assert!(response.is_ok());
//     assert_eq!(response.unwrap().into_inner().instances.len(), 1);
//
//     // Shutdown the server
//     shutdown_tx.send(()).ok();
//     Ok(())
// }

use hypervisor_connector_proxmox::mock::{MockServer, WithClusterResourceList};
use instances::{
    Instance,
    v1::{ListInstancesRequest, instances_client::InstancesClient},
};
use resources::DEFAULT_PROJECT_NAME;
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_instances_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new().await.with_cluster_resource_list();
    let mock_url = mock.url();

    Instance::factory()
        .for_hypervisor_with(|hypervisor| hypervisor.url(mock_url))
        .for_project_with(|project| {
            project
                .name(DEFAULT_PROJECT_NAME.into())
                .for_organization_with(|organization| organization)
        })
        .create(pool.clone())
        .await?;

    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = InstancesClient::connect(format!("http://{}", addr)).await?;

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
