use spicedb::SpiceDB;
use workflow::execution::WorkflowExecutionId;
use workflow::operations::Operation;
use workflow::operations::check_permission::{CheckPermissionError, CheckPermissionOp};

mod common;

fn sample_op() -> CheckPermissionOp {
    CheckPermissionOp {
        subject_type: "user".to_owned(),
        subject_id: "alice".to_owned(),
        permission: "create_instance".to_owned(),
        resource_type: "project".to_owned(),
        resource_id: "my-project".to_owned(),
    }
}

#[tokio::test]
async fn execute_succeeds_when_permission_is_granted() {
    let spicedb = SpiceDB::mock().await;
    let ctx = common::context_with_spicedb(spicedb);

    let result = sample_op().execute(ctx, WorkflowExecutionId::new()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn execute_returns_forbidden_when_permission_is_denied() {
    let spicedb = SpiceDB::denying().await;
    let ctx = common::context_with_spicedb(spicedb);

    let err = sample_op()
        .execute(ctx, WorkflowExecutionId::new())
        .await
        .unwrap_err();

    assert!(matches!(err, CheckPermissionError::Forbidden));
}

#[tokio::test]
async fn forbidden_error_is_an_invariant_violation() {
    use workflow::operations::OperationError;

    let spicedb = SpiceDB::denying().await;
    let ctx = common::context_with_spicedb(spicedb);

    let err = sample_op()
        .execute(ctx, WorkflowExecutionId::new())
        .await
        .unwrap_err();

    assert!(err.is_violated_invariant());
}

#[tokio::test]
async fn rollback_is_a_noop() {
    let spicedb = SpiceDB::mock().await;
    let ctx = common::context_with_spicedb(spicedb);

    let result = sample_op().rollback(ctx, WorkflowExecutionId::new()).await;

    assert!(result.is_ok());
}
