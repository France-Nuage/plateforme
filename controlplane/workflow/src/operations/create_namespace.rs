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

        match namespaces.create(&PostParams::default(), &ns).await {
            Ok(_) => info!(namespace = %self.namespace, "namespace created"),
            Err(KubeError::Api(ref err)) if err.code == 409 => {
                info!(namespace = %self.namespace, "namespace already exists");
            }
            Err(e) => return Err(e.into()),
        }

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let namespaces = Api::<Namespace>::all(ctx.kube);

        match namespaces
            .delete(&self.namespace, &Default::default())
            .await
        {
            Ok(_) => info!(namespace = %self.namespace, "namespace deleted (rollback)"),
            Err(KubeError::Api(ref err)) if err.code == 404 => {
                info!(namespace = %self.namespace, "namespace already gone (rollback)");
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }
}
