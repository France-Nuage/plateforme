use resources::v1::{
    CreateOrganizationRequest, CreateOrganizationResponse, Organization,
    resources_client::ResourcesClient,
};
use server::{Server, ServerConfig};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_organization_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new(pool.clone());
    let server = Server::new(config).await?;
    let addr = server.addr;
    let shutdown_tx = server.serve_with_shutdown().await?;
    let mut client = ResourcesClient::connect(format!("http://{}", addr)).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .create_organization(CreateOrganizationRequest {
            name: String::from("ACME"),
        })
        .await;

    // Get the instance generated in the database
    let organization = &resources::organizations::repository::list(&pool)
        .await
        .unwrap()[0];

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateOrganizationResponse {
            organization: Some(Organization {
                id: organization.id.to_string(),
                name: String::from("ACME"),
                created_at: Some(prost_types::Timestamp::default()),
                updated_at: Some(prost_types::Timestamp::default()),
            })
        }
    );

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
