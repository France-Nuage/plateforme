use crate::common::{Api, OnBehalfOf};
use fabrique::Persist;
use frn_core::longrunning::Operation;
use frn_rpc::v1::longrunning::WaitOperationRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_wait_blocks_until_operation_completes(pool: sqlx::PgPool) {
    let mut api = Api::start(&pool).await.expect("could not start api");

    // Arrange: create a pending operation
    let operation = Operation::write_relationships(vec![])
        .expect("could not create operation")
        .create(&pool)
        .await
        .expect("could not persist operation");

    let operation_id = operation.id;
    let pool_clone = pool.clone();

    // Spawn a task to complete the operation after a short delay
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Operation::mark_completed(operation_id, &pool_clone)
            .await
            .expect("could not mark operation completed");
    });

    // Act: call the Wait RPC
    let request = Request::new(WaitOperationRequest {
        name: operation.id.to_string(),
        timeout: None,
    })
    .on_behalf_of(&api.service_account);

    let response = api.longrunning.operations.wait(request).await;

    // Assert
    assert!(response.is_ok());
    let op = response.unwrap().into_inner();
    assert_eq!(op.name, operation.id.to_string());
    assert_eq!(op.done, Some(true));
}
