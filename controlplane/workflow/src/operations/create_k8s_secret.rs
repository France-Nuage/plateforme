use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Secret;
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
pub struct CreateK8sSecretOp {
    pub namespace: String,
    pub secret_name: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum CreateK8sSecretError {
    #[error("kubernetes API error: {0}")]
    #[operation_error(transient)]
    Kube(#[from] KubeError),
}

impl crate::operations::Operation for CreateK8sSecretOp {
    type Error = CreateK8sSecretError;

    async fn execute(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let secrets = Api::<Secret>::namespaced(ctx.kube, &self.namespace);

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(self.secret_name.clone()),
                namespace: Some(self.namespace.clone()),
                ..Default::default()
            },
            string_data: Some(self.data.clone()),
            ..Default::default()
        };

        match classify_create(secrets.create(&PostParams::default(), &secret).await)? {
            CreateOutcome::Created => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret created"
            ),
            CreateOutcome::AlreadyExists => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret already exists"
            ),
        }

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let secrets = Api::<Secret>::namespaced(ctx.kube, &self.namespace);

        match classify_delete(secrets.delete(&self.secret_name, &Default::default()).await)? {
            DeleteOutcome::Deleted => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret deleted (rollback)"
            ),
            DeleteOutcome::AlreadyGone => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret already gone (rollback)"
            ),
        }

        Ok(())
    }
}
