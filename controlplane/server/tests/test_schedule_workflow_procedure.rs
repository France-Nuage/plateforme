use crate::common::{Api, IntoWorker};
use frn_rpc::v1::workflow::{ExecutionStatus, Initiator, ScheduleRequest, initiator};
use serde_json::json;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_schedule_workflow_returns_execution(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let definition = json!({"WriteRelationships": {"relationships": [], "done": false}});

    let request = Request::new(ScheduleRequest {
        definition: definition.to_string(),
        max_retry: 3,
        initiated_by: Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        }),
        schedule_at: None,
    })
    .into_worker();

    let response = api.workflow.engine.schedule(request).await;

    assert!(response.is_ok());
    let resp = response.unwrap().into_inner();
    assert!(resp.execution.is_some());

    let execution = resp.execution.unwrap();
    assert_eq!(execution.status, ExecutionStatus::Pending as i32);
    assert_eq!(execution.max_try_count, 3);
    assert_eq!(execution.soft_try_count, 0);
    assert_eq!(execution.hard_try_count, 0);
    assert!(!execution.execution_id.is_empty());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_schedule_workflow_rejects_invalid_definition(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(ScheduleRequest {
        definition: "not valid json".to_owned(),
        max_retry: 3,
        initiated_by: Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        }),
        schedule_at: None,
    })
    .into_worker();

    let response = api.workflow.engine.schedule(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_schedule_workflow_rejects_unauthenticated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(ScheduleRequest {
        definition: "{}".to_owned(),
        max_retry: 3,
        initiated_by: Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        }),
        schedule_at: None,
    });

    let response = api.workflow.engine.schedule(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_schedule_workflow_rejects_wrong_token(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let mut request = Request::new(ScheduleRequest {
        definition: "{}".to_owned(),
        max_retry: 3,
        initiated_by: Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        }),
        schedule_at: None,
    });
    request
        .metadata_mut()
        .insert("authorization", "Bearer wrong-token".parse()?);

    let response = api.workflow.engine.schedule(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);

    Ok(())
}
