use auth::mock::WithWellKnown;
use database::Persistable;
use frn_rpc::v1::resourcemanager::{
    CreateOrganizationRequest, CreateOrganizationResponse, Organization,
    organizations_client::OrganizationsClient,
};
use mock_server::MockServer;
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_organization_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mock = MockServer::new().await.with_well_known();

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = OrganizationsClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client
        .create(CreateOrganizationRequest {
            name: String::from("ACME"),
        })
        .await;

    // Get the instance generated in the database
    let organizations = frn_core::resourcemanager::Organization::list(&pool)
        .await
        .unwrap();
    println!("orgs: {:#?}", &organizations);

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateOrganizationResponse {
            organization: Some(Organization {
                id: organizations[0].id.to_string(),
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
