use serde::{Deserialize, Serialize};
use spicedb::ObjectRef;
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

/// Verifies that a principal still holds a specific permission on a resource.
///
/// Used as a defense-in-depth barrier inside workflows: even though the API
/// layer already checked the permission before scheduling the workflow, this
/// operation re-checks at execution time so that a revoked permission or a
/// directly-injected workflow row cannot bypass authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckPermissionOp {
    pub subject_type: String,
    pub subject_id: String,
    pub permission: String,
    pub resource_type: String,
    pub resource_id: String,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum CheckPermissionError {
    #[error("permission denied")]
    #[operation_error(invariant)]
    Forbidden,

    #[error("spicedb error: {0}")]
    #[operation_error(transient)]
    SpiceDb(spicedb::Error),
}

impl crate::operations::Operation for CheckPermissionOp {
    type Error = CheckPermissionError;

    async fn execute(
        self,
        mut ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let subject = ObjectRef::new(&self.subject_type, &self.subject_id);
        let resource = ObjectRef::new(&self.resource_type, &self.resource_id);

        ctx.spicedb
            .check_permission(subject, self.permission.clone(), resource)
            .await
            .map_err(|e| match e {
                spicedb::Error::Forbidden => CheckPermissionError::Forbidden,
                other => CheckPermissionError::SpiceDb(other),
            })?;

        info!(
            subject_type = %self.subject_type,
            subject_id = %self.subject_id,
            permission = %self.permission,
            resource_type = %self.resource_type,
            resource_id = %self.resource_id,
            "permission check passed"
        );

        Ok(self)
    }

    async fn rollback(
        self,
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
