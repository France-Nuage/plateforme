use crate::common::{Api, OnBehalfOf};
use fabrique::Factory;
use frn_core::resourcemanager::Organization;
use frn_rpc::v1::resourcemanager::ListOrganizationsRequest;
use spicedb::mock::AUTHORIZED_RESOURCE_ID;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_organizations_procedure_works(pool: sqlx::PgPool) {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // The mock SpiceDB only authorizes AUTHORIZED_RESOURCE_ID, and list filters
    // by lookup, so the seeded organization must carry that slug to be returned.
    Organization::factory()
        .slug(AUTHORIZED_RESOURCE_ID.to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not create organization");
    // Act the request to the test_the_status_procedure_works
    let request =
        Request::new(ListOrganizationsRequest::default()).on_behalf_of(&api.service_account);

    let response = api.resourcemanager.organizations.list(request).await;

    // Assert the result
    println!("response: {:#?}", &response);
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().organizations.len(), 1);
}
