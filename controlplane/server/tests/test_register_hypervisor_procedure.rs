use crate::common::{Api, OnBehalfOf};
use fabrique::Factory;
use frn_core::{compute::Zone, resourcemanager::Organization};
use frn_rpc::v1::compute::RegisterHypervisorRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_register_hypervisor_procedure_works(pool: sqlx::PgPool) {
    let mut api = Api::start(&pool).await.expect("count not start api");
    let zone = Zone::factory()
        .create(&pool)
        .await
        .expect("could not create zone");
    let organization = Organization::factory()
        .parent_id(None)
        .create(&pool)
        .await
        .expect("could not create organization");

    // Act the request to the test_the_status_procedure_works
    let request = Request::new(RegisterHypervisorRequest {
        zone_id: zone.id.to_string(),
        organization_id: organization.id.to_string(),
        ..Default::default()
    })
    .on_behalf_of(&api.service_account);
    let result = api.compute.hypervisors.register(request).await;

    // Assert the result
    assert!(result.is_ok());
}
