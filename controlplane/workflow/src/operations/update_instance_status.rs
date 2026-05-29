use frn_core::managed::{ManagedServiceError, ManagedServices};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;
use crate::fsm::{FsmRepository, TransitionError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstanceStatusOp {
    pub instance_id: Uuid,
    pub new_status: String,
    /// Populated during execute to allow rollback.
    pub previous_status: Option<String>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum UpdateInstanceStatusError {
    #[error("{0}")]
    ManagedService(#[from] ManagedServiceError),

    #[error("invalid status transition: {0}")]
    #[operation_error(invariant)]
    InvalidTransition(String),

    #[error("FSM database error: {0}")]
    #[operation_error(transient)]
    FsmDatabase(SqlxError),
}

impl crate::operations::Operation for UpdateInstanceStatusOp {
    type Error = UpdateInstanceStatusError;

    async fn execute(
        mut self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let managed = ManagedServices::new(
            ctx.spicedb.clone(),
            ctx.pool.clone(),
            ctx.platform_config.clone(),
        );
        let instance = managed.find_instance(self.instance_id).await?;

        let mut conn = ctx
            .pool
            .acquire()
            .await
            .map_err(UpdateInstanceStatusError::FsmDatabase)?;

        let current_name = FsmRepository::current_state_name(&mut conn, &instance.status)
            .await
            .map_err(UpdateInstanceStatusError::FsmDatabase)?;

        self.previous_status = Some(current_name);

        FsmRepository::state_machine_transition(
            &mut conn,
            &instance.status,
            self.new_status.clone(),
        )
        .await
        .map_err(|e| match e {
            TransitionError::Database(db_err) => UpdateInstanceStatusError::FsmDatabase(db_err),
            TransitionError::InvalidTransition(from, to, available) => {
                UpdateInstanceStatusError::InvalidTransition(format!(
                    "cannot transition from {from} to {to}, available: {available}"
                ))
            }
        })?;

        info!(
            instance_id = %self.instance_id,
            status = %self.new_status,
            "instance status updated"
        );

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let Some(previous) = self.previous_status else {
            info!(
                instance_id = %self.instance_id,
                "no previous status captured, skipping rollback"
            );
            return Ok(());
        };

        let managed = ManagedServices::new(
            ctx.spicedb.clone(),
            ctx.pool.clone(),
            ctx.platform_config.clone(),
        );
        let instance = managed.find_instance(self.instance_id).await?;

        let mut conn = ctx
            .pool
            .acquire()
            .await
            .map_err(UpdateInstanceStatusError::FsmDatabase)?;

        match FsmRepository::state_machine_transition(&mut conn, &instance.status, previous.clone())
            .await
        {
            Ok(()) => {
                info!(
                    instance_id = %self.instance_id,
                    status = %previous,
                    "instance status restored (rollback)"
                );
            }
            Err(TransitionError::InvalidTransition(..)) => {
                info!(
                    instance_id = %self.instance_id,
                    "reverse FSM transition not available, skipping rollback"
                );
            }
            Err(TransitionError::Database(e)) => {
                return Err(UpdateInstanceStatusError::FsmDatabase(e));
            }
        }

        Ok(())
    }
}
