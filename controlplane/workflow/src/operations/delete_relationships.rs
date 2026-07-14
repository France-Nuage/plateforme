use frn_core::authorization::Relationship;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRelationshipsOp {
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum DeleteRelationshipsError {
    #[error("spicedb error: {0}")]
    SpiceDb(#[from] spicedb::Error),
}

impl crate::operations::Operation for DeleteRelationshipsOp {
    type Error = DeleteRelationshipsError;

    async fn execute(
        self,
        mut ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let batch = self.relationships.iter().map(Into::into).collect();

        ctx.spicedb.delete_relationships(batch).await?;

        for relationship in &self.relationships {
            info!("deleted relationship: {relationship}");
        }

        Ok(self)
    }

    async fn rollback(
        self,
        mut ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let batch = self.relationships.into_iter().map(Into::into).collect();

        ctx.spicedb.write_relationships(batch).await?;

        Ok(())
    }
}
