use crate::common::{Api, OnBehalfOf};
use frn_core::{compute::Instance, resourcemanager::Organization};
use frn_rpc::v1::compute::StopInstanceRequest;
use sqlx::types::Uuid;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_start_instance_procedure_works(pool: sqlx::PgPool) {
    // Arrange the grpc server and a client
    let mut api = Api::start(&pool).await.expect("could not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .expect("could not create organization");
    let instance = Instance::factory()
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .for_default_zone()
                .for_default_organization()
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .distant_id("100".into())
        .create(&pool)
        .await
        .expect("could not create instance");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(StopInstanceRequest {
        id: instance.id.to_string(),
    })
    .on_behalf_of(&api.service_account);
    let response = api.compute.instances.stop(request).await;

    // Assert the result
    assert!(response.is_ok());
}
