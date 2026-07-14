use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Api;
use kube::Error as KubeError;
use kube::api::PostParams;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;
use crate::operations::k8s_common::{
    CreateOutcome, DeleteOutcome, classify_create, classify_delete,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamespaceOp {
    pub namespace: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum CreateNamespaceError {
    #[error("kubernetes API error: {0}")]
    #[operation_error(transient)]
    Kube(#[from] KubeError),
}

impl crate::operations::Operation for CreateNamespaceOp {
    type Error = CreateNamespaceError;

    async fn execute(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let namespaces = Api::<Namespace>::all(ctx.kube);

        let ns = Namespace {
            metadata: ObjectMeta {
                name: Some(self.namespace.clone()),
                labels: Some(self.labels.clone()),
                ..Default::default()
            },
            ..Default::default()
        };

        match classify_create(namespaces.create(&PostParams::default(), &ns).await)? {
            CreateOutcome::Created => info!(namespace = %self.namespace, "namespace created"),
            CreateOutcome::AlreadyExists => {
                info!(namespace = %self.namespace, "namespace already exists")
            }
        }

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let namespaces = Api::<Namespace>::all(ctx.kube);

        match classify_delete(
            namespaces
                .delete(&self.namespace, &Default::default())
                .await,
        )? {
            DeleteOutcome::Deleted => {
                info!(namespace = %self.namespace, "namespace deleted (rollback)")
            }
            DeleteOutcome::AlreadyGone => {
                info!(namespace = %self.namespace, "namespace already gone (rollback)")
            }
        }

        Ok(())
    }
}
