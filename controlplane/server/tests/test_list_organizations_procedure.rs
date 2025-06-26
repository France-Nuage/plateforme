use resources::{
    organizations::Organization,
    v1::{ListOrganizationsRequest, resources_client::ResourcesClient},
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    Organization::factory().create(&pool).await?;
    let config = ServerConfig::new(pool);
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = ResourcesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .list_organizations(ListOrganizationsRequest::default())
        .await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().organizations.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
