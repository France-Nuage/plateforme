use crate::common::{Api, OnBehalfOf};
use fabrique::Factory;
use frn_core::{
    compute::{Hypervisor, Instance, Zone},
    resourcemanager::{Organization, Project},
};
use frn_rpc::v1::compute::StartInstanceRequest;
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
    let hypervisor = Hypervisor::factory()
        .for_zone(Zone::factory())
        .organization_id(organization.id)
        .url(mock_url)
        .create(&pool)
        .await
        .expect("could not create hypervisor");
    let project = Project::factory()
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create project");
    let instance = Instance::factory()
        .hypervisor_id(hypervisor.id)
        .project_id(project.id)
        .distant_id("100".into())
        .create(&pool)
        .await
        .expect("could not create instance");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(StartInstanceRequest {
        id: instance.id.to_string(),
    })
    .on_behalf_of(&api.service_account);
    let response = api.compute.instances.start(request).await;

    // Assert the result
    assert!(response.is_ok());
}
