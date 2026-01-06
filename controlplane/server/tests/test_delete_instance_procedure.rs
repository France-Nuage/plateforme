use crate::common::{Api, OnBehalfOf};
use fabrique::Factory;
use frn_core::{
    compute::{Hypervisor, Instance, Zone},
    resourcemanager::{Organization, Project},
};
use frn_rpc::v1::compute::DeleteInstanceRequest;
use tonic::Request;
mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_delete_instance_procedure_works(pool: sqlx::PgPool) {
    // Arrange the grpc server and a client
    let mut api = Api::start(&pool).await.expect("count not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .parent_id(None)
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
        .distant_id("100".into())
        .hypervisor_id(hypervisor.id)
        .project_id(project.id)
        .zero_trust_network_id(None)
        .create(&pool)
        .await
        .expect("could not create instance");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(DeleteInstanceRequest {
        id: instance.id.to_string(),
    })
    .on_behalf_of(&api.service_account);
    let response = api.compute.instances.delete(request).await;

    // Assert the result
    assert!(response.is_ok());
}
