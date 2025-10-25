use frn_core::resourcemanager::Organization;
use instances::{Instance, v1::CloneInstanceRequest};
use tonic::Request;

use crate::common::{Api, OnBehalfOf};

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_clone_instance_procedure_works(pool: sqlx::PgPool) {
    // Arrange the grpc server and a client
    let mut api = Api::start(&pool).await.expect("count not start api");
    let mock_url = api.mock_server.url();
    let organization = Organization::factory()
        .create(&pool)
        .await
        .expect("could not create organization");
    let instance = Instance::factory()
        .distant_id("100".into())
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .url(mock_url)
                .for_default_zone()
                .organization_id(organization.id)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .create(&pool)
        .await
        .expect("could not create instance");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(CloneInstanceRequest {
        id: instance.id.to_string(),
    })
    .on_behalf_of(&api.service_account);
    let response = api.compute.instances.clone_instance(request).await;

    // Assert the result
    assert!(response.is_ok());
}
