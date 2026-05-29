use crate::common::{Api, IntoWorker};
use frn_rpc::v1::workflow::{ExecutionStatus, Initiator, NextRequest, ScheduleRequest, initiator};
use serde_json::json;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_next_returns_empty_when_no_executions(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(NextRequest {}).into_worker();
    let response = api.workflow.engine.next(request).await;

    assert!(response.is_ok());
    let resp = response.unwrap().into_inner();
    assert!(resp.execution.is_none());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_returns_pending_execution(
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

    let schedule_resp = api
        .workflow
        .engine
        .schedule(schedule_request)
        .await?
        .into_inner();
    let scheduled_id = schedule_resp.execution.unwrap().execution_id;

    let next_request = Request::new(NextRequest {}).into_worker();
    let next_resp = api.workflow.engine.next(next_request).await;

    assert!(next_resp.is_ok());
    let execution = next_resp.unwrap().into_inner().execution;
    assert!(execution.is_some());

    let execution = execution.unwrap();
    assert_eq!(execution.execution_id, scheduled_id);
    assert_eq!(execution.status, ExecutionStatus::Running as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_rejects_unauthenticated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(NextRequest {});
    let response = api.workflow.engine.next(request).await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::Unauthenticated);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_locks_execution_so_second_call_returns_empty(
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

    let first = Request::new(NextRequest {}).into_worker();
    let first_resp = api.workflow.engine.next(first).await?.into_inner();
    assert!(first_resp.execution.is_some());

    let second = Request::new(NextRequest {}).into_worker();
    let second_resp = api.workflow.engine.next(second).await?.into_inner();
    assert!(second_resp.execution.is_none());

    Ok(())
}
