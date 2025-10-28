use crate::common::{Api, OnBehalfOf};
use database::Persistable;
use frn_rpc::v1::resourcemanager::{
    CreateOrganizationRequest, CreateOrganizationResponse, Organization,
};
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_organization_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("count not start api");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(CreateOrganizationRequest {
        name: String::from("ACME"),
        parent_id: None,
    })
    .on_behalf_of(&api.service_account);
    let response = api.resourcemanager.organizations.create(request).await;

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
    Ok(())
}
