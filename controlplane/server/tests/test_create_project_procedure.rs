use resources::{
    organizations::Organization,
    v1::{CreateProjectRequest, CreateProjectResponse, Project, resources_client::ResourcesClient},
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_project_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let config = ServerConfig::new(pool.clone());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = ResourcesClient::connect(format!("http://{}", addr)).await?;

    let organization = Organization::factory().create(pool.clone()).await?;

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
