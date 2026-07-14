use crate::auth::authenticate_bearer;
pub use crate::timestamp::{from_timestamp, to_timestamp};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use std::fmt::Display;
use tonic::{Request, Response, Status};
use workflow::execution::{
    WorkflowExecution as WfExecution, WorkflowExecutionId, WorkflowExecutionStatus,
    WorkflowInitiator,
};
use workflow::repository::WorkflowExecutionError;
use workflow::service::WorkflowService;
use workflow::workflows::WorkflowDefinitions;

tonic::include_proto!("francenuage.fr.v1.workflow");

pub struct WorkflowEngine {
    pool: Pool<Postgres>,
    worker_token: String,
}

impl WorkflowEngine {
    pub fn new(pool: Pool<Postgres>, worker_token: String) -> Self {
        Self { pool, worker_token }
    }

    fn authenticate(&self, request: &Request<impl Sized>) -> Result<(), Status> {
        authenticate_bearer(request, &self.worker_token, "invalid worker token")
    }
}

impl From<WorkflowExecutionStatus> for ExecutionStatus {
    fn from(status: WorkflowExecutionStatus) -> Self {
        match status {
            WorkflowExecutionStatus::Pending => Self::Pending,
            WorkflowExecutionStatus::Running => Self::Running,
            WorkflowExecutionStatus::WillRetry => Self::WillRetry,
            WorkflowExecutionStatus::Completed => Self::Completed,
            WorkflowExecutionStatus::Failed => Self::Failed,
        }
    }
}

impl From<&WorkflowInitiator> for Initiator {
    fn from(initiator: &WorkflowInitiator) -> Self {
        match initiator {
            WorkflowInitiator::User(id) => Self {
                kind: Some(initiator::Kind::UserId(id.to_string())),
            },
            WorkflowInitiator::Workflow(id) => Self {
                kind: Some(initiator::Kind::WorkflowId(id.to_string())),
            },
            WorkflowInitiator::System => Self {
                kind: Some(initiator::Kind::System(true)),
            },
        }
    }
}

pub fn status_from_proto(value: i32) -> Result<WorkflowExecutionStatus, Status> {
    match ExecutionStatus::try_from(value) {
        Ok(ExecutionStatus::Pending) => Ok(WorkflowExecutionStatus::Pending),
        Ok(ExecutionStatus::Running) => Ok(WorkflowExecutionStatus::Running),
        Ok(ExecutionStatus::WillRetry) => Ok(WorkflowExecutionStatus::WillRetry),
        Ok(ExecutionStatus::Completed) => Ok(WorkflowExecutionStatus::Completed),
        Ok(ExecutionStatus::Failed) => Ok(WorkflowExecutionStatus::Failed),
        Ok(ExecutionStatus::Unspecified) | Err(_) => {
            Err(Status::invalid_argument(format!("invalid status: {value}")))
        }
    }
}

pub fn initiator_from_proto(initiator: Option<Initiator>) -> Result<WorkflowInitiator, Status> {
    let init = initiator.ok_or_else(|| Status::invalid_argument("missing initiated_by"))?;
    match init.kind {
        Some(initiator::Kind::UserId(id)) => {
            let uuid = id
                .parse()
                .map_err(|_| Status::invalid_argument("invalid user_id in initiator"))?;
            Ok(WorkflowInitiator::User(uuid))
        }
        Some(initiator::Kind::WorkflowId(id)) => {
            let wf_id: WorkflowExecutionId = id
                .parse()
                .map_err(|_| Status::invalid_argument("invalid workflow_id in initiator"))?;
            Ok(WorkflowInitiator::Workflow(wf_id))
        }
        Some(initiator::Kind::System(_)) => Ok(WorkflowInitiator::System),
        None => Err(Status::invalid_argument("empty initiator")),
    }
}

impl TryFrom<&WfExecution> for WorkflowExecution {
    type Error = serde_json::Error;

    fn try_from(exec: &WfExecution) -> Result<Self, Self::Error> {
        Ok(Self {
            execution_id: exec.execution_id.to_string(),
            initiated_by: Some((&exec.initiated_by).into()),
            status: ExecutionStatus::from(exec.status).into(),
            soft_try_count: exec.soft_try_count,
            hard_try_count: exec.hard_try_count,
            max_try_count: exec.max_try_count,
            next_retry_at: Some(to_timestamp(exec.next_retry_at)),
            dependencies: exec.dependencies.iter().map(|d| d.to_string()).collect(),
            definition: serde_json::to_string(&exec.definition)?,
        })
    }
}

impl TryFrom<WorkflowExecution> for WfExecution {
    type Error = Status;

    fn try_from(proto: WorkflowExecution) -> Result<Self, Self::Error> {
        let execution_id: WorkflowExecutionId = proto
            .execution_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid execution_id"))?;

        let initiated_by = initiator_from_proto(proto.initiated_by)?;
        let status = status_from_proto(proto.status)?;

        let next_retry_at = proto
            .next_retry_at
            .as_ref()
            .map(from_timestamp)
            .transpose()?
            .unwrap_or_else(Utc::now);

        let dependencies: Result<Vec<WorkflowExecutionId>, _> =
            proto.dependencies.iter().map(|d| d.parse()).collect();
        let dependencies =
            dependencies.map_err(|_| Status::invalid_argument("invalid dependency id"))?;

        let definition: WorkflowDefinitions = serde_json::from_str(&proto.definition)
            .map_err(|e| Status::invalid_argument(format!("invalid definition: {e}")))?;

        Ok(Self {
            execution_id,
            initiated_by,
            status,
            soft_try_count: proto.soft_try_count,
            hard_try_count: proto.hard_try_count,
            max_try_count: proto.max_try_count,
            next_retry_at,
            dependencies,
            definition,
        })
    }
}

/// Logs an internal failure and returns a generic `Status` so raw database or
/// transaction errors never leak to the caller.
fn internal_status(context: &str, err: impl Display) -> Status {
    tracing::error!(context = %context, error = %err, "workflow engine internal error");
    Status::internal("internal error")
}

fn workflow_error_to_status(err: WorkflowExecutionError) -> Status {
    match err {
        WorkflowExecutionError::Database(_) => internal_status("workflow repository", err),
        WorkflowExecutionError::InvalidTransition(_) => {
            Status::failed_precondition(err.to_string())
        }
        WorkflowExecutionError::JsonError(_) => Status::invalid_argument(err.to_string()),
    }
}

#[tonic::async_trait]
impl workflow_engine_server::WorkflowEngine for WorkflowEngine {
    async fn next(&self, request: Request<NextRequest>) -> Result<Response<NextResponse>, Status> {
        self.authenticate(&request)?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| internal_status("begin transaction", e))?;

        let execution = WorkflowService::fetch_and_lock_next_workflow_execution(&mut tx)
            .await
            .map_err(|e| internal_status("fetch next workflow", e))?;

        tx.commit()
            .await
            .map_err(|e| internal_status("commit transaction", e))?;

        let proto_execution = execution
            .as_ref()
            .map(WorkflowExecution::try_from)
            .transpose()
            .map_err(|e| internal_status("serialize execution", e))?;

        Ok(Response::new(NextResponse {
            execution: proto_execution,
        }))
    }

    async fn schedule(
        &self,
        request: Request<ScheduleRequest>,
    ) -> Result<Response<ScheduleResponse>, Status> {
        self.authenticate(&request)?;

        let req = request.into_inner();

        let definition: WorkflowDefinitions = serde_json::from_str(&req.definition)
            .map_err(|e| Status::invalid_argument(format!("invalid definition: {e}")))?;

        let initiated_by = initiator_from_proto(req.initiated_by)?;
        let schedule_at = req.schedule_at.as_ref().map(from_timestamp).transpose()?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| internal_status("begin transaction", e))?;

        let execution = WorkflowService::schedule_workflow(
            &mut tx,
            definition,
            req.max_retry,
            initiated_by,
            schedule_at,
        )
        .await
        .map_err(workflow_error_to_status)?;

        tx.commit()
            .await
            .map_err(|e| internal_status("commit transaction", e))?;

        let proto_execution = WorkflowExecution::try_from(&execution)
            .map_err(|e| internal_status("serialize execution", e))?;

        Ok(Response::new(ScheduleResponse {
            execution: Some(proto_execution),
        }))
    }

    async fn unlock(
        &self,
        request: Request<UnlockRequest>,
    ) -> Result<Response<UnlockResponse>, Status> {
        self.authenticate(&request)?;

        let proto_exec = request
            .into_inner()
            .execution
            .ok_or_else(|| Status::invalid_argument("missing execution"))?;

        let execution: WfExecution = proto_exec.try_into()?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| internal_status("begin transaction", e))?;

        WorkflowService::unlock_workflow_execution(&mut tx, execution)
            .await
            .map_err(workflow_error_to_status)?;

        tx.commit()
            .await
            .map_err(|e| internal_status("commit transaction", e))?;

        Ok(Response::new(UnlockResponse {}))
    }

    async fn get_status(
        &self,
        request: Request<GetStatusRequest>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        self.authenticate(&request)?;

        let execution_id: WorkflowExecutionId = request
            .into_inner()
            .execution_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid execution_id"))?;

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| internal_status("acquire connection", e))?;

        let status = WorkflowService::fetch_status(&mut conn, execution_id)
            .await
            .map_err(|e| internal_status("fetch workflow status", e))?;

        Ok(Response::new(GetStatusResponse {
            status: ExecutionStatus::from(status.status).into(),
            next_retry_at: Some(to_timestamp(status.next_retry_at)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use prost_types::Timestamp;
    use uuid::Uuid;
    use workflow::workflows::write_relationships::WriteRelationshipsWorkflow;

    fn sample_definition() -> WorkflowDefinitions {
        WorkflowDefinitions::WriteRelationships(WriteRelationshipsWorkflow::new(vec![]))
    }

    fn valid_proto() -> WorkflowExecution {
        WorkflowExecution {
            execution_id: WorkflowExecutionId::new().to_string(),
            initiated_by: Some(Initiator {
                kind: Some(initiator::Kind::System(true)),
            }),
            status: ExecutionStatus::Pending as i32,
            soft_try_count: 1,
            hard_try_count: 2,
            max_try_count: 3,
            next_retry_at: Some(Timestamp {
                seconds: 1_700_000_000,
                nanos: 123_456_789,
            }),
            dependencies: vec![],
            definition: serde_json::to_string(&sample_definition()).unwrap(),
        }
    }

    #[test]
    fn status_from_proto_maps_every_known_variant() {
        let mapped: Vec<WorkflowExecutionStatus> = [
            ExecutionStatus::Pending,
            ExecutionStatus::Running,
            ExecutionStatus::WillRetry,
            ExecutionStatus::Completed,
            ExecutionStatus::Failed,
        ]
        .into_iter()
        .map(|s| status_from_proto(s as i32).unwrap())
        .collect();

        assert_eq!(
            mapped,
            vec![
                WorkflowExecutionStatus::Pending,
                WorkflowExecutionStatus::Running,
                WorkflowExecutionStatus::WillRetry,
                WorkflowExecutionStatus::Completed,
                WorkflowExecutionStatus::Failed,
            ]
        );
    }

    #[test]
    fn status_from_proto_rejects_unspecified() {
        let result = status_from_proto(ExecutionStatus::Unspecified as i32);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn status_from_proto_rejects_out_of_range_value() {
        let result = status_from_proto(999);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn initiator_from_proto_reads_user() {
        let uuid = Uuid::new_v4();
        let proto = Some(Initiator {
            kind: Some(initiator::Kind::UserId(uuid.to_string())),
        });

        let result = initiator_from_proto(proto);

        assert!(matches!(result, Ok(WorkflowInitiator::User(u)) if u == uuid));
    }

    #[test]
    fn initiator_from_proto_reads_workflow() {
        let id = WorkflowExecutionId::new();
        let proto = Some(Initiator {
            kind: Some(initiator::Kind::WorkflowId(id.to_string())),
        });

        let result = initiator_from_proto(proto);

        assert!(matches!(result, Ok(WorkflowInitiator::Workflow(w)) if w == id));
    }

    #[test]
    fn initiator_from_proto_reads_system() {
        let proto = Some(Initiator {
            kind: Some(initiator::Kind::System(true)),
        });

        let result = initiator_from_proto(proto);

        assert!(matches!(result, Ok(WorkflowInitiator::System)));
    }

    #[test]
    fn initiator_from_proto_rejects_malformed_user_id() {
        let proto = Some(Initiator {
            kind: Some(initiator::Kind::UserId("not-a-uuid".to_owned())),
        });

        let result = initiator_from_proto(proto);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn initiator_from_proto_rejects_missing_initiator() {
        let result = initiator_from_proto(None);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn initiator_from_proto_rejects_empty_kind() {
        let result = initiator_from_proto(Some(Initiator { kind: None }));

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn execution_round_trips_through_proto_without_losing_fields() {
        let domain = WfExecution {
            execution_id: WorkflowExecutionId::new(),
            initiated_by: WorkflowInitiator::User(Uuid::new_v4()),
            status: WorkflowExecutionStatus::WillRetry,
            soft_try_count: 4,
            hard_try_count: 2,
            max_try_count: 7,
            next_retry_at: Utc
                .timestamp_opt(1_700_000_000, 123_456_789)
                .single()
                .unwrap(),
            dependencies: vec![WorkflowExecutionId::new(), WorkflowExecutionId::new()],
            definition: sample_definition(),
        };

        let proto = WorkflowExecution::try_from(&domain).unwrap();
        let restored = WfExecution::try_from(proto).unwrap();

        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&domain).unwrap()
        );
    }

    #[test]
    fn execution_try_from_rejects_malformed_execution_id() {
        let proto = WorkflowExecution {
            execution_id: "not-a-uuid".to_owned(),
            ..valid_proto()
        };

        let result = WfExecution::try_from(proto);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn execution_try_from_rejects_malformed_dependency_id() {
        let proto = WorkflowExecution {
            dependencies: vec!["not-a-uuid".to_owned()],
            ..valid_proto()
        };

        let result = WfExecution::try_from(proto);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn execution_try_from_rejects_undeserializable_definition() {
        let proto = WorkflowExecution {
            definition: "{\"NotAWorkflow\": {}}".to_owned(),
            ..valid_proto()
        };

        let result = WfExecution::try_from(proto);

        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }
}
