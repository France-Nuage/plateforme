use crate::common::{Api, OnBehalfOf};
use fabrique::Factory;
use frn_core::resourcemanager::{Organization, Project};
use frn_rpc::v1::resourcemanager::ListProjectsRequest;
use sqlx::types::Uuid;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_projects_procedure_works(pool: sqlx::PgPool) {
    // Arrange the test
    let mut api = Api::start(&pool).await.expect("could not start api");

    let organization = Organization::factory()
        .parent_id(None)
        .create(&pool)
        .await
        .expect("could not create organization");
    Project::factory()
        .id(Uuid::default())
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create project");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(ListProjectsRequest::default()).on_behalf_of(&api.service_account);
    let response = api.resourcemanager.projects.list(request).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().projects.len(), 1);
}
