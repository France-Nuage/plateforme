use k8s_openapi::api::core::v1::Namespace;
use kube::Api;
use kube::Error as KubeError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

/// Asserts that a Kubernetes namespace does not yet exist.
///
/// Used in the create-only deploy workflow to fail fast if the target namespace
/// is already present, which would indicate either a naming collision or a
/// stale resource from a previous deployment that was not cleaned up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertNamespaceAbsentOp {
    pub namespace: String,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum AssertNamespaceAbsentError {
    #[error("namespace {0} already exists")]
    #[operation_error(invariant)]
    AlreadyExists(String),

    #[error("kubernetes API error: {0}")]
    #[operation_error(transient)]
    Kube(#[from] KubeError),
}

impl crate::operations::Operation for AssertNamespaceAbsentOp {
    type Error = AssertNamespaceAbsentError;

    async fn execute(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let namespaces = Api::<Namespace>::all(ctx.kube);

        match namespaces.get_opt(&self.namespace).await? {
            None => {
                info!(namespace = %self.namespace, "namespace absent, safe to create");
                Ok(self)
            }
            Some(_) => Err(AssertNamespaceAbsentError::AlreadyExists(
                self.namespace.clone(),
            )),
        }
    }

    async fn rollback(
        self,
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
