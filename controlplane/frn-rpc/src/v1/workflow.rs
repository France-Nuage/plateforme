use crate::auth::authenticate_bearer;
pub use crate::timestamp::{from_timestamp, to_timestamp};
use chrono::Utc;
use sqlx::{Pool, Postgres};
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

fn workflow_error_to_status(err: WorkflowExecutionError) -> Status {
    match err {
        WorkflowExecutionError::Database(_) => Status::internal(err.to_string()),
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
            .map_err(|e| Status::internal(e.to_string()))?;

        let execution = WorkflowService::fetch_and_lock_next_workflow_execution(&mut tx)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_execution = execution
            .as_ref()
            .map(WorkflowExecution::try_from)
            .transpose()
            .map_err(|e| Status::internal(format!("serialization error: {e}")))?;

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
            .map_err(|e| Status::internal(e.to_string()))?;

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
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_execution = WorkflowExecution::try_from(&execution)
            .map_err(|e| Status::internal(format!("serialization error: {e}")))?;

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
            .map_err(|e| Status::internal(e.to_string()))?;

        WorkflowService::unlock_workflow_execution(&mut tx, execution)
            .await
            .map_err(workflow_error_to_status)?;

        tx.commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

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
            .map_err(|e| Status::internal(e.to_string()))?;

        let status = WorkflowService::fetch_status(&mut conn, execution_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetStatusResponse {
            status: ExecutionStatus::from(status.status).into(),
            next_retry_at: Some(to_timestamp(status.next_retry_at)),
        }))
    }
}
