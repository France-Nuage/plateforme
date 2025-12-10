use crate::common::{Api, OnBehalfOf};
use frn_core::resourcemanager::Project;
use frn_rpc::v1::resourcemanager::ListProjectsRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_projects_procedure_works(pool: sqlx::PgPool) {
    // Arrange the test
    let mut api = Api::start(&pool).await.expect("could not start api");

    Project::factory()
        .for_organization(|organization| organization)
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
