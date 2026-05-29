use frn_core::authorization::Relationship;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRelationshipsOp {
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum WriteRelationshipsError {
    #[error("spicedb error: {0}")]
    SpiceDb(#[from] spicedb::Error),
}

impl crate::operations::Operation for WriteRelationshipsOp {
    type Error = WriteRelationshipsError;

    async fn execute(
        self,
        mut ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let batch = self
            .relationships
            .iter()
            .map(|rel| {
                (
                    rel.subject_type.clone(),
                    rel.subject_id.clone(),
                    rel.relation.to_string(),
                    rel.object_type.clone(),
                    rel.object_id.clone(),
                )
            })
            .collect();

        ctx.spicedb.write_relationships(batch).await?;

        for relationship in &self.relationships {
            info!("wrote relationship: {relationship}");
        }

        Ok(self)
    }

    async fn rollback(
        self,
        mut ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let batch = self
            .relationships
            .into_iter()
            .rev()
            .map(|rel| {
                (
                    rel.subject_type,
                    rel.subject_id,
                    rel.relation.to_string(),
                    rel.object_type,
                    rel.object_id,
                )
            })
            .collect();

        ctx.spicedb.delete_relationships(batch).await?;

        Ok(())
    }
}
