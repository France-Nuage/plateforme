use crate::common::{Api, IntoWorker};
use frn_rpc::v1::workflow::{
    ExecutionStatus, GetStatusRequest, Initiator, ScheduleRequest, initiator,
};
use serde_json::json;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_returns_pending_for_new_workflow(
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
    let execution_id = schedule_resp.execution.unwrap().execution_id;

    let status_request = Request::new(GetStatusRequest {
        execution_id: execution_id.clone(),
    })
    .into_worker();
    let status_resp = api
        .workflow
        .engine
        .get_status(status_request)
        .await?
        .into_inner();

    assert_eq!(status_resp.status, ExecutionStatus::Pending as i32);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_rejects_invalid_execution_id(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(GetStatusRequest {
        execution_id: "not-a-uuid".to_owned(),
    })
    .into_worker();

    let response = api.workflow.engine.get_status(request).await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_status_rejects_unauthenticated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let request = Request::new(GetStatusRequest {
        execution_id: "00000000-0000-0000-0000-000000000000".to_owned(),
    });

    let response = api.workflow.engine.get_status(request).await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::Unauthenticated);

    Ok(())
}
