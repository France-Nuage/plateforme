use fabrique::{Factory, Query};
use frn_core::resourcemanager::Organization;
use frn_rpc::v1::resourcemanager::{CreateProjectRequest, CreateProjectResponse, Project};
mod common;
use common::Api;
use tonic::Request;

use crate::common::OnBehalfOf;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_project_procedure_works(pool: sqlx::PgPool) {
    // Arrange the test
    let mut api = Api::start(&pool).await.expect("count not start api");

    let organization = Organization::factory()
        .parent_id(None)
        .create(&pool)
        .await
        .expect("could not create organization");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(CreateProjectRequest {
        name: String::from("ACME"),
        organization_id: organization.id.to_string(),
    })
    .on_behalf_of(&api.service_account);
    let response = api.resourcemanager.projects.create(request).await;

    // Get the project generated in the database
    let projects = frn_core::resourcemanager::Project::all(&pool)
        .await
        .unwrap();

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().into_inner(),
        CreateProjectResponse {
            project: Some(Project {
                id: projects[0].id.to_string(),
                name: String::from("ACME"),
                organization_id: organization.id.to_string(),
                created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    projects[0].created_at
                ))),
                updated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    projects[0].updated_at
                ))),
            })
        }
    );
}
