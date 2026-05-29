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

        match secrets.create(&PostParams::default(), &secret).await {
            Ok(_) => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret created"
            ),
            Err(KubeError::Api(ref err)) if err.code == 409 => {
                info!(
                    namespace = %self.namespace,
                    secret = %self.secret_name,
                    "secret already exists"
                );
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
        let secrets = Api::<Secret>::namespaced(ctx.kube, &self.namespace);

        match secrets.delete(&self.secret_name, &Default::default()).await {
            Ok(_) => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret deleted (rollback)"
            ),
            Err(KubeError::Api(ref err)) if err.code == 404 => {
                info!(
                    namespace = %self.namespace,
                    secret = %self.secret_name,
                    "secret already gone (rollback)"
                );
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }
}
