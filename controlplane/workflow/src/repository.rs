//! Persistence for workflow executions.
//!
//! Data access here uses raw `sqlx` rather than the `fabrique` builder: every
//! read and write is coupled to the `lib_fsm` state-machine schema (status is a
//! `lib_fsm.state_machine__id` resolved through joins on
//! `lib_fsm.abstract_state`, transitions go through the
//! `lib_fsm.state_machine_transition` SQL function, and the poller relies on a
//! `FOR UPDATE SKIP LOCKED` CTE). None of these are expressible with the query
//! builder, so this crate intentionally does not depend on `fabrique`. Each
//! query below notes the specific construct that forces raw SQL.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgConnection;
use sqlx::types::Json;
use thiserror::Error;
use tracing::warn;
use uuid::Uuid;

use crate::execution::{
    WorkflowExecution, WorkflowExecutionId, WorkflowExecutionStatus, WorkflowInitiator,
};
use crate::fsm::{FsmRepository, TransitionError};
use crate::workflows::WorkflowDefinitions;

const LOCK_DURATION_MINUTES: i32 = 5;

#[derive(sqlx::FromRow)]
struct WorkflowExecutionRow {
    execution_id: WorkflowExecutionId,
    initiated_by_user: Option<Uuid>,
    initiated_by_workflow: Option<Uuid>,
    soft_try_count: i32,
    hard_try_count: i32,
    max_try_count: i32,
    definition: Json<WorkflowDefinitions>,
    next_retry_at: DateTime<Utc>,
    status: WorkflowExecutionStatus,
    dependencies: Vec<WorkflowExecutionId>,
}

pub struct WorkflowExecutionRepository;

#[derive(Debug, Error)]
pub enum WorkflowExecutionError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid transition: {0}")]
    InvalidTransition(String),
    #[error("JSON (de)serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl From<TransitionError> for WorkflowExecutionError {
    fn from(err: TransitionError) -> Self {
        match err {
            TransitionError::Database(error) => Self::Database(error),
            TransitionError::InvalidTransition(_, _, _) => Self::InvalidTransition(err.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FetchWorkflowStatus {
    pub status: WorkflowExecutionStatus,
    pub next_retry_at: DateTime<Utc>,
}

impl WorkflowExecutionRepository {
    pub async fn find_by_idempotency_key(
        conn: &mut PgConnection,
        key: &str,
    ) -> Result<Option<WorkflowExecution>, WorkflowExecutionError> {
        // Raw SQL: this crate is sqlx-only (see module docs); no fabrique models exist for workflow.execution.
        let row: Option<(WorkflowExecutionId,)> = sqlx::query_as(
            "SELECT execution_id FROM workflow.execution WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&mut *conn)
        .await?;

        match row {
            Some((id,)) => Ok(Some(Self::fetch_one(&mut *conn, id).await?)),
            None => Ok(None),
        }
    }

    pub async fn create(
        conn: &mut PgConnection,
        workflow: WorkflowExecution,
        idempotency_key: Option<&str>,
    ) -> Result<WorkflowExecution, WorkflowExecutionError> {
        let (initiated_by_user, initiated_by_workflow) = match workflow.initiated_by {
            WorkflowInitiator::User(user_id) => (Some(user_id), None),
            WorkflowInitiator::Workflow(workflow_id) => (None, Some(workflow_id.as_uuid())),
            WorkflowInitiator::System => (None, None),
        };

        // Raw SQL: RETURNING the lib_fsm status id to feed state_machine_transition; not expressible with fabrique.
        // ON CONFLICT DO NOTHING makes concurrent inserts sharing an idempotency_key race-safe: the loser
        // gets no row back (DO NOTHING raises no error, so the transaction stays usable) and returns the
        // already-committed execution below instead of surfacing a raw unique-constraint violation. The
        // conflict target matches the partial unique index (WHERE idempotency_key IS NOT NULL), so NULL
        // keys never conflict and always insert.
        let status: Option<Uuid> = sqlx::query_scalar(
            r#"INSERT INTO workflow.execution
                (execution_id, initiated_by_user, initiated_by_workflow,
                 soft_try_count, hard_try_count, max_try_count, definition, next_retry_at,
                 idempotency_key)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               ON CONFLICT (idempotency_key) WHERE idempotency_key IS NOT NULL DO NOTHING
               RETURNING status"#,
        )
        .bind(workflow.execution_id.as_uuid())
        .bind(initiated_by_user)
        .bind(initiated_by_workflow)
        .bind(workflow.soft_try_count)
        .bind(workflow.hard_try_count)
        .bind(workflow.max_try_count)
        .bind(serde_json::to_value(&workflow.definition)?)
        .bind(workflow.next_retry_at)
        .bind(idempotency_key)
        .fetch_optional(&mut *conn)
        .await?;

        let Some(_status) = status else {
            // Lost the idempotency race. ON CONFLICT waits on the winning insert, so by the time we see
            // no returned row the winner has committed and its execution is visible to this read.
            let key = idempotency_key
                .expect("a conflict on idempotency_key can only happen when the key is set");
            return Self::find_by_idempotency_key(&mut *conn, key)
                .await?
                .ok_or(WorkflowExecutionError::Database(sqlx::Error::RowNotFound));
        };

        debug_assert_eq!(
            workflow.status,
            WorkflowExecutionStatus::Pending,
            "WorkflowExecution::new() must produce Pending status"
        );

        for dependency in &workflow.dependencies {
            Self::create_dependency(&mut *conn, workflow.execution_id, *dependency).await?;
        }

        Ok(workflow)
    }

    async fn create_dependency(
        conn: &mut PgConnection,
        execution_id: WorkflowExecutionId,
        dependency: WorkflowExecutionId,
    ) -> Result<(), sqlx::Error> {
        // Raw SQL: this crate is sqlx-only (see module docs); no fabrique model for execution_dependency.
        sqlx::query(
            r#"INSERT INTO workflow.execution_dependency (execution_id, dependency_id)
               VALUES ($1, $2)"#,
        )
        .bind(execution_id.as_uuid())
        .bind(dependency.as_uuid())
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn fetch_and_lock_next_workflow(
        conn: &mut PgConnection,
    ) -> Result<Option<WorkflowExecution>, TransitionError> {
        // Raw SQL: FOR UPDATE SKIP LOCKED CTE plus lib_fsm joins; not expressible with fabrique.
        let row: Option<(WorkflowExecutionId, Uuid, WorkflowExecutionStatus)> = sqlx::query_as(
            r#"WITH job AS (
                    SELECT execution_id
                    FROM workflow.execution exec
                    INNER JOIN lib_fsm.state_machine sm ON exec.status = sm.state_machine__id
                    INNER JOIN lib_fsm.abstract_state abs ON sm.abstract_state__id = abs.abstract_state__id
                    WHERE abs.name IN ('pending', 'will_retry', 'running')
                      AND next_retry_at <= now()
                      AND (locked_until <= now() OR locked_until IS NULL)
                    ORDER BY next_retry_at ASC
                    LIMIT 1
                    FOR UPDATE SKIP LOCKED
                )
                UPDATE workflow.execution exec
                SET locked_until = now() + $1 * interval '1 minute'
                FROM job
                WHERE exec.execution_id = job.execution_id
                RETURNING exec.execution_id, exec.status, (
                    SELECT abs.name FROM lib_fsm.state_machine sm
                    INNER JOIN lib_fsm.abstract_state abs ON sm.abstract_state__id = abs.abstract_state__id
                    WHERE sm.state_machine__id = exec.status
                )"#,
        )
        .bind(LOCK_DURATION_MINUTES)
        .fetch_optional(&mut *conn)
        .await?;

        let Some((id, status, status_name)) = row else {
            return Ok(None);
        };

        if status_name != WorkflowExecutionStatus::Running {
            FsmRepository::state_machine_transition(
                &mut *conn,
                &status,
                WorkflowExecutionStatus::Running.to_string(),
            )
            .await?;
        }

        Ok(Some(Self::fetch_one(&mut *conn, id).await?))
    }

    pub async fn fetch_one(
        conn: &mut PgConnection,
        id: WorkflowExecutionId,
    ) -> Result<WorkflowExecution, sqlx::Error> {
        // Raw SQL: lib_fsm joins to resolve the status name and an array_agg subquery for dependencies.
        let row: WorkflowExecutionRow = sqlx::query_as(
            r#"SELECT
                    execution_id,
                    initiated_by_user,
                    initiated_by_workflow,
                    soft_try_count,
                    hard_try_count,
                    max_try_count,
                    definition,
                    next_retry_at,
                    abs.name AS status,
                    (SELECT coalesce(array_agg(dependency_id), ARRAY[]::UUID[])
                     FROM workflow.execution_dependency dep
                     WHERE dep.execution_id = $1) AS dependencies
                FROM workflow.execution exec
                INNER JOIN lib_fsm.state_machine sm ON sm.state_machine__id = exec.status
                INNER JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id
                WHERE execution_id = $1"#,
        )
        .bind(id.as_uuid())
        .fetch_one(&mut *conn)
        .await?;

        Ok(WorkflowExecution {
            execution_id: row.execution_id,
            initiated_by: match (row.initiated_by_user, row.initiated_by_workflow) {
                (Some(user), None) => WorkflowInitiator::User(user),
                (None, Some(workflow)) => {
                    WorkflowInitiator::Workflow(WorkflowExecutionId::from_uuid(workflow))
                }
                (Some(_user), Some(workflow)) => {
                    warn!("workflow execution initiated by both user and workflow");
                    WorkflowInitiator::Workflow(WorkflowExecutionId::from_uuid(workflow))
                }
                (None, None) => WorkflowInitiator::System,
            },
            status: row.status,
            soft_try_count: row.soft_try_count,
            hard_try_count: row.hard_try_count,
            max_try_count: row.max_try_count,
            next_retry_at: row.next_retry_at,
            dependencies: row.dependencies,
            definition: row.definition.0,
        })
    }

    pub async fn unlock(
        conn: &mut PgConnection,
        execution: &WorkflowExecution,
    ) -> Result<(), WorkflowExecutionError> {
        // Raw SQL: RETURNING the lib_fsm status id to feed state_machine_transition; not expressible with fabrique.
        let status: Uuid = sqlx::query_scalar(
            r#"UPDATE workflow.execution
               SET soft_try_count = $1, hard_try_count = $2, max_try_count = $3,
                   definition = $4, next_retry_at = $5, locked_until = NULL
               WHERE execution_id = $6
               RETURNING status"#,
        )
        .bind(execution.soft_try_count)
        .bind(execution.hard_try_count)
        .bind(execution.max_try_count)
        .bind(serde_json::to_value(&execution.definition)?)
        .bind(execution.next_retry_at)
        .bind(execution.execution_id.as_uuid())
        .fetch_one(&mut *conn)
        .await?;

        FsmRepository::state_machine_transition(conn, &status, execution.status.to_string())
            .await?;

        // Raw SQL: this crate is sqlx-only (see module docs); no fabrique model for execution_dependency.
        sqlx::query("DELETE FROM workflow.execution_dependency WHERE execution_id = $1")
            .bind(execution.execution_id.as_uuid())
            .execute(&mut *conn)
            .await?;

        for dependency in &execution.dependencies {
            Self::create_dependency(&mut *conn, execution.execution_id, *dependency).await?;
        }

        Ok(())
    }

    pub async fn fetch_status(
        conn: &mut PgConnection,
        execution_id: WorkflowExecutionId,
    ) -> Result<FetchWorkflowStatus, TransitionError> {
        // Raw SQL: lib_fsm joins to resolve the status name plus a GREATEST() over next_retry_at/locked_until.
        let row: (WorkflowExecutionStatus, DateTime<Utc>) = sqlx::query_as(
            r#"SELECT abs.name AS status,
                      GREATEST(exec.next_retry_at, coalesce(exec.locked_until, now())) AS next_retry_at
               FROM workflow.execution exec
               INNER JOIN lib_fsm.state_machine sm ON sm.state_machine__id = exec.status
               INNER JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id
               WHERE execution_id = $1"#,
        )
        .bind(execution_id.as_uuid())
        .fetch_one(&mut *conn)
        .await?;

        Ok(FetchWorkflowStatus {
            status: row.0,
            next_retry_at: row.1,
        })
    }
}
