use fabrique::Factory;
use frn_core::{
    compute::{Hypervisor, Instance, Zone},
    resourcemanager::{Organization, Project},
};
use frn_rpc::v1::compute::CloneInstanceRequest;
use tonic::Request;

use crate::common::{Api, OnBehalfOf};

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_clone_instance_procedure_works(pool: sqlx::PgPool) {
    // Arrange the grpc server and a client
    let mut api = Api::start(&pool).await.expect("count not start api");
    let mock_url = api.mock_server.url();
    let organization = Organization::factory()
        .slug("test-org".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not create organization");
    let hypervisor = Hypervisor::factory()
        .url(mock_url)
        .for_zone(Zone::factory())
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await
        .expect("could not create hypervisor");
    let project = Project::factory()
        .slug("test-project".to_owned())
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await
        .expect("could not create project");
    let instance = Instance::factory()
        .distant_id("100".into())
        .hypervisor_id(hypervisor.id)
        .project_slug(project.slug.clone())
        .zero_trust_network_id(None)
        .create(&pool)
        .await
        .expect("could not create instance");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(CloneInstanceRequest {
        id: instance.id.to_string(),
        name: None,
    })
    .on_behalf_of(&api.service_account);

    let instances = &mut api.compute.instances;
    let response = instances.clone(request).await;

    // Assert the result
    assert!(response.is_ok());
}
