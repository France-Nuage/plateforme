use frn_core::managed::{ManagedServiceError, ManagedServices};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstanceVersionOp {
    pub instance_id: Uuid,
    pub version_id: Uuid,
    /// Populated during execute to allow rollback.
    pub previous_version_id: Option<Uuid>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum UpdateInstanceVersionError {
    #[error("{0}")]
    ManagedService(#[from] ManagedServiceError),
}

impl crate::operations::Operation for UpdateInstanceVersionOp {
    type Error = UpdateInstanceVersionError;

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
        self.previous_version_id = Some(instance.version_id);

        managed
            .update_instance_version(self.instance_id, self.version_id)
            .await?;

        info!(
            instance_id = %self.instance_id,
            version_id = %self.version_id,
            "instance version updated"
        );

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let Some(previous) = self.previous_version_id else {
            info!(
                instance_id = %self.instance_id,
                "no previous version captured, skipping rollback"
            );
            return Ok(());
        };

        let managed = ManagedServices::new(
            ctx.spicedb.clone(),
            ctx.pool.clone(),
            ctx.platform_config.clone(),
        );
        managed
            .update_instance_version(self.instance_id, previous)
            .await?;

        info!(
            instance_id = %self.instance_id,
            version_id = %previous,
            "instance version restored (rollback)"
        );

        Ok(())
    }
}
