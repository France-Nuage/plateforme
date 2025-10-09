use auth::mock::WithWellKnown;
use frn_core::resourcemanager::Project;
use frn_rpc::v1::resourcemanager::{ListProjectsRequest, projects_client::ProjectsClient};
use mock_server::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_projects_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await.with_well_known();

    Project::factory()
        .for_organization_with(|organization| organization)
        .create(&pool)
        .await?;

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = ProjectsClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client.list(ListProjectsRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().projects.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
