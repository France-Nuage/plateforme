use crate::common::{Api, OnBehalfOf};
use frn_core::compute::Hypervisor;
use frn_rpc::v1::compute::DetachHypervisorRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_detach_hypervisor_procedure_works(pool: sqlx::PgPool) {
    // Arrange the grpc server and a client
    let hypervisor = Hypervisor::factory()
        .for_default_zone()
        .for_organization_with(|organization| organization)
        .create(&pool)
        .await
        .expect("could not bootstrap data");

    let mut api = Api::start(&pool).await.expect("could not start api");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(DetachHypervisorRequest {
        id: hypervisor.id.to_string(),
    })
    .on_behalf_of(&api.service_account);
    let result = api.compute.hypervisors.detach(request).await;

    // Assert the result
    assert!(result.is_ok());
}
