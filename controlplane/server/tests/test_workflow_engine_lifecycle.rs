use crate::common::{Api, IntoWorker};
use chrono::Utc;
use frn_rpc::v1::workflow::{
    ExecutionStatus, GetStatusRequest, Initiator, NextRequest, ScheduleRequest, UnlockRequest,
    initiator, to_timestamp,
};
use serde_json::json;
use tonic::Request;

mod common;

/// A minimal, deserializable workflow definition used across the lifecycle
/// tests. `WriteRelationships` with an empty relationship set touches no
/// external system when executed.
fn write_relationships_definition() -> String {
    json!({ "WriteRelationships": { "relationships": [], "done": false } }).to_string()
}

fn system_initiator() -> Option<Initiator> {
    Some(Initiator {
        kind: Some(initiator::Kind::System(true)),
    })
}

async fn schedule(
    api: &mut Api,
    initiated_by: Option<Initiator>,
    schedule_at: Option<prost_types::Timestamp>,
) -> frn_rpc::v1::workflow::WorkflowExecution {
    let request = Request::new(ScheduleRequest {
        definition: write_relationships_definition(),
        max_retry: 3,
        initiated_by,
        schedule_at,
    })
    .into_worker();

    api.workflow
        .engine
        .schedule(request)
        .await
        .expect("schedule must succeed")
        .into_inner()
        .execution
        .expect("schedule response must contain an execution")
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_returns_none_when_no_workflow_is_due(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("next must succeed")
        .into_inner();

    assert!(response.execution.is_none());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_returns_scheduled_workflow_as_running(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let scheduled = schedule(&mut api, system_initiator(), None).await;

    let picked = api
        .workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("next must succeed")
        .into_inner()
        .execution
        .expect("next must return the due execution");

    assert_eq!(
        (picked.execution_id, picked.status),
        (scheduled.execution_id, ExecutionStatus::Running as i32)
    );

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_skips_a_locked_workflow(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    schedule(&mut api, system_initiator(), None).await;

    api.workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("first next must succeed");

    let second = api
        .workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("second next must succeed")
        .into_inner();

    assert!(second.execution.is_none());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_ignores_a_workflow_scheduled_in_the_future(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let future = to_timestamp(Utc::now() + chrono::Duration::hours(1));
    schedule(&mut api, system_initiator(), Some(future)).await;

    let response = api
        .workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("next must succeed")
        .into_inner();

    assert!(response.execution.is_none());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_returns_pending_after_scheduling(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let scheduled = schedule(&mut api, system_initiator(), None).await;

    let response = api
        .workflow
        .engine
        .get_status(
            Request::new(GetStatusRequest {
                execution_id: scheduled.execution_id,
            })
            .into_worker(),
        )
        .await
        .expect("get_status must succeed")
        .into_inner();

    assert_eq!(response.status, ExecutionStatus::Pending as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_returns_running_after_next(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let scheduled = schedule(&mut api, system_initiator(), None).await;

    api.workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("next must succeed");

    let response = api
        .workflow
        .engine
        .get_status(
            Request::new(GetStatusRequest {
                execution_id: scheduled.execution_id,
            })
            .into_worker(),
        )
        .await
        .expect("get_status must succeed")
        .into_inner();

    assert_eq!(response.status, ExecutionStatus::Running as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_rejects_invalid_execution_id(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .workflow
        .engine
        .get_status(
            Request::new(GetStatusRequest {
                execution_id: "not-a-uuid".to_owned(),
            })
            .into_worker(),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_of_unknown_execution_errors(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .workflow
        .engine
        .get_status(
            Request::new(GetStatusRequest {
                execution_id: uuid::Uuid::now_v7().to_string(),
            })
            .into_worker(),
        )
        .await;

    assert!(response.is_err());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_unlock_persists_a_completed_status(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let scheduled = schedule(&mut api, system_initiator(), None).await;

    let mut running = api
        .workflow
        .engine
        .next(Request::new(NextRequest {}).into_worker())
        .await
        .expect("next must succeed")
        .into_inner()
        .execution
        .expect("next must return the execution");
    running.status = ExecutionStatus::Completed as i32;

    api.workflow
        .engine
        .unlock(
            Request::new(UnlockRequest {
                execution: Some(running),
            })
            .into_worker(),
        )
        .await
        .expect("unlock must succeed");

    let status = api
        .workflow
        .engine
        .get_status(
            Request::new(GetStatusRequest {
                execution_id: scheduled.execution_id,
            })
            .into_worker(),
        )
        .await
        .expect("get_status must succeed")
        .into_inner();

    assert_eq!(status.status, ExecutionStatus::Completed as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_scheduling_the_same_sub_workflow_twice_is_deduplicated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let parent = schedule(&mut api, system_initiator(), None).await;
    let parent_initiator = Some(Initiator {
        kind: Some(initiator::Kind::WorkflowId(parent.execution_id)),
    });

    let first = schedule(&mut api, parent_initiator.clone(), None).await;
    let second = schedule(&mut api, parent_initiator, None).await;

    assert_eq!(first.execution_id, second.execution_id);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_next_rejects_unauthenticated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api.workflow.engine.next(Request::new(NextRequest {})).await;

    assert_eq!(response.unwrap_err().code(), tonic::Code::Unauthenticated);

    Ok(())
}
