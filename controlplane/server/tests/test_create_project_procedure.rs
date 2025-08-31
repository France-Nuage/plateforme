use mock_server::MockServer;
use resources::{
    organizations::Organization,
    v1::{CreateProjectRequest, CreateProjectResponse, Project, resources_client::ResourcesClient},
};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_project_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await;

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = ResourcesClient::connect(server_url).await?;

    let organization = Organization::factory().create(&pool).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .create_project(CreateProjectRequest {
            name: String::from("ACME"),
            organization_id: organization.id.to_string(),
        })
        .await;

    // Get the project generated in the database
    let project = &resources::projects::repository::list(&pool).await.unwrap()[0];

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateProjectResponse {
            project: Some(Project {
                id: project.id.to_string(),
                name: String::from("ACME"),
                organization_id: organization.id.to_string(),
                created_at: Some(prost_types::Timestamp::default()),
                updated_at: Some(prost_types::Timestamp::default()),
            })
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
