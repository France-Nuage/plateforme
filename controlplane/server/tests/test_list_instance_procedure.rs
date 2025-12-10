use crate::common::{Api, OnBehalfOf};
use frn_core::compute::Instance;
use frn_core::resourcemanager::{DEFAULT_PROJECT_NAME, Organization};
use frn_rpc::v1::compute::ListInstancesRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_instances_procedure_works(pool: sqlx::PgPool) {
    // Arrange the grpc server and a client
    let mut api = Api::start(&pool).await.expect("could not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .create(&pool)
        .await
        .expect("could not create organization");
    Instance::factory()
        .for_hypervisor(move |hypervisor| {
            hypervisor
                .for_zone(|zone| zone)
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project(move |project| {
            project
                .name(DEFAULT_PROJECT_NAME.into())
                .organization_id(organization.id)
        })
        .create(&pool)
        .await
        .expect("could not create instance");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(ListInstancesRequest::default()).on_behalf_of(&api.service_account);
    let response = api.compute.instances.list(request).await;

    // Assert the result
    assert!(response.is_ok());
    let instances = response.unwrap().into_inner().instances;
    assert_eq!(instances.len(), 1);
}
