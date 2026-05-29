use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::execution::{
    WorkflowExecution, WorkflowExecutionId, WorkflowExecutionStatus, WorkflowInitiator,
};
use crate::fsm::TransitionError;
use crate::repository::{FetchWorkflowStatus, WorkflowExecutionError, WorkflowExecutionRepository};
use crate::workflows::WorkflowDefinitions;

const IDEMPOTENCY_NAMESPACE: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

pub struct WorkflowService;

impl WorkflowService {
    fn compute_idempotency_key(
        parent_id: WorkflowExecutionId,
        definition: &WorkflowDefinitions,
    ) -> Result<String, serde_json::Error> {
        let serialized = serde_json::to_string(definition)?;
        let input = format!("{}:{}", parent_id, serialized);
        Ok(Uuid::new_v5(&IDEMPOTENCY_NAMESPACE, input.as_bytes()).to_string())
    }

    pub async fn schedule_workflow(
        conn: &mut sqlx::PgConnection,
        workflow: WorkflowDefinitions,
        max_retry: i32,
        initiated_by: WorkflowInitiator,
        schedule_at: Option<DateTime<Utc>>,
    ) -> Result<WorkflowExecution, WorkflowExecutionError> {
        let idempotency_key = match initiated_by {
            WorkflowInitiator::Workflow(parent_id) => {
                Some(Self::compute_idempotency_key(parent_id, &workflow)?)
            }
            _ => None,
        };

        if let Some(ref key) = idempotency_key
            && let Some(existing) =
                WorkflowExecutionRepository::find_by_idempotency_key(conn, key).await?
        {
            tracing::info!(
                execution_id = %existing.execution_id,
                "deduplicated sub-workflow, returning existing execution"
            );
            return Ok(existing);
        }

        let execution = WorkflowExecutionRepository::create(
            conn,
            WorkflowExecution::new(initiated_by, max_retry, workflow, schedule_at),
            idempotency_key.as_deref(),
        )
        .await?;

        tracing::info!(
            execution_id = %execution.execution_id,
            initiated_by = ?initiated_by,
            "workflow execution created"
        );

        Ok(execution)
    }

    pub async fn fetch_and_lock_next_workflow_execution(
        conn: &mut sqlx::PgConnection,
    ) -> Result<Option<WorkflowExecution>, TransitionError> {
        WorkflowExecutionRepository::fetch_and_lock_next_workflow(conn).await
    }

    pub async fn unlock_workflow_execution(
        conn: &mut sqlx::PgConnection,
        execution: WorkflowExecution,
    ) -> Result<(), WorkflowExecutionError> {
        WorkflowExecutionRepository::unlock(&mut *conn, &execution).await?;

        tracing::info!(
            execution_id = %execution.execution_id,
            workflow = %execution.definition.name(),
            status = ?execution.status,
            "workflow execution updated"
        );

        if execution.status == WorkflowExecutionStatus::Failed {
            tracing::error!(
                execution_id = %execution.execution_id,
                "workflow execution failed, notification pending"
            );
        }

        Ok(())
    }

    pub async fn fetch_status(
        conn: &mut sqlx::PgConnection,
        execution_id: WorkflowExecutionId,
    ) -> Result<FetchWorkflowStatus, TransitionError> {
        WorkflowExecutionRepository::fetch_status(&mut *conn, execution_id).await
    }
}
