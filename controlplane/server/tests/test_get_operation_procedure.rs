use crate::common::{Api, OnBehalfOf};
use fabrique::Persist;
use frn_core::longrunning::Operation;
use frn_rpc::v1::longrunning::GetOperationRequest;
use sqlx::types::chrono::Utc;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_get_operation_procedure_works(pool: sqlx::PgPool) {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // Arrange: create an operation in the database
    let operation = Operation::write_relationships(vec![])
        .expect("could not create operation")
        .create(&pool)
        .await
        .expect("could not persist operation");

    // Act: call the Get RPC
    let request = Request::new(GetOperationRequest {
        name: operation.id.to_string(),
    })
    .on_behalf_of(&api.service_account);

    let response = api.longrunning.operations.get(request).await;

    // Assert the result
    println!("response: {:#?}", &response);
    assert!(response.is_ok());
    let op = response.unwrap().into_inner();
    assert_eq!(op.name, operation.id.to_string());
    assert_eq!(op.done, Some(false));
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_get_operation_returns_done_when_completed(pool: sqlx::PgPool) {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // Arrange: create a completed operation in the database
    let mut operation = Operation::write_relationships(vec![]).expect("could not create operation");
    operation.completed_at = Some(Utc::now());
    let operation = operation
        .create(&pool)
        .await
        .expect("could not persist operation");

    // Act: call the Get RPC
    let request = Request::new(GetOperationRequest {
        name: operation.id.to_string(),
    })
    .on_behalf_of(&api.service_account);

    let response = api.longrunning.operations.get(request).await;

    // Assert the result
    assert!(response.is_ok());
    let op = response.unwrap().into_inner();
    assert_eq!(op.name, operation.id.to_string());
    assert_eq!(op.done, Some(true));
}
