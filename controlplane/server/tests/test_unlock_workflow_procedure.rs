use crate::common::{Api, IntoWorker};
use frn_rpc::v1::workflow::{
    ExecutionStatus, GetStatusRequest, Initiator, NextRequest, ScheduleRequest, UnlockRequest,
    initiator,
};
use serde_json::json;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_unlock_marks_workflow_completed(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let definition = json!({"WriteRelationships": {"relationships": [], "done": false}});

    let schedule_request = Request::new(ScheduleRequest {
        definition: definition.to_string(),
        max_retry: 3,
        initiated_by: Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        }),
        schedule_at: None,
    })
    .into_worker();
    api.workflow.engine.schedule(schedule_request).await?;

    let next_request = Request::new(NextRequest {}).into_worker();
    let mut execution = api
        .workflow
        .engine
        .next(next_request)
        .await?
        .into_inner()
        .execution
        .expect("should have an execution");

    execution.status = ExecutionStatus::Completed as i32;

    let unlock_request = Request::new(UnlockRequest {
        execution: Some(execution.clone()),
    })
    .into_worker();
    let unlock_resp = api.workflow.engine.unlock(unlock_request).await;
    assert!(unlock_resp.is_ok());

    let status_request = Request::new(GetStatusRequest {
        execution_id: execution.execution_id.clone(),
    })
    .into_worker();
    let status_resp = api
        .workflow
        .engine
        .get_status(status_request)
        .await?
        .into_inner();
    assert_eq!(status_resp.status, ExecutionStatus::Completed as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_unlock_marks_workflow_failed(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let definition = json!({"WriteRelationships": {"relationships": [], "done": false}});

    let schedule_request = Request::new(ScheduleRequest {
        definition: definition.to_string(),
        max_retry: 3,
        initiated_by: Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        }),
        schedule_at: None,
    })
    .into_worker();
    api.workflow.engine.schedule(schedule_request).await?;

    let next_request = Request::new(NextRequest {}).into_worker();
    let mut execution = api
        .workflow
        .engine
        .next(next_request)
        .await?
        .into_inner()
        .execution
        .expect("should have an execution");

    execution.status = ExecutionStatus::Failed as i32;
    execution.hard_try_count = 3;

    let unlock_request = Request::new(UnlockRequest {
        execution: Some(execution.clone()),
    })
    .into_worker();
    let unlock_resp = api.workflow.engine.unlock(unlock_request).await;
    assert!(unlock_resp.is_ok());

    let status_request = Request::new(GetStatusRequest {
        execution_id: execution.execution_id.clone(),
    })
    .into_worker();
    let status_resp = api
        .workflow
        .engine
        .get_status(status_request)
        .await?
        .into_inner();
    assert_eq!(status_resp.status, ExecutionStatus::Failed as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_unlock_rejects_unauthenticated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(UnlockRequest { execution: None });
    let response = api.workflow.engine.unlock(request).await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::Unauthenticated);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_unlock_rejects_missing_execution(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(UnlockRequest { execution: None }).into_worker();
    let response = api.workflow.engine.unlock(request).await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);

    Ok(())
}
