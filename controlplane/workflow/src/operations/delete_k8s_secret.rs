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

/// Secret data is captured from K8s during execute to allow rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteK8sSecretOp {
    pub namespace: String,
    pub secret_name: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum DeleteK8sSecretError {
    #[error("kubernetes API error: {0}")]
    #[operation_error(transient)]
    Kube(#[from] KubeError),
}

impl crate::operations::Operation for DeleteK8sSecretOp {
    type Error = DeleteK8sSecretError;

    async fn execute(
        mut self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let secrets: Api<Secret> = Api::namespaced(ctx.kube, &self.namespace);

        match secrets.get(&self.secret_name).await {
            Ok(secret) => {
                if let Some(data) = secret.data {
                    self.data = data
                        .into_iter()
                        .filter_map(|(k, v)| String::from_utf8(v.0).ok().map(|s| (k, s)))
                        .collect();
                }
            }
            Err(KubeError::Api(ref err)) if err.code == 404 => {}
            Err(e) => return Err(e.into()),
        }

        match secrets.delete(&self.secret_name, &Default::default()).await {
            Ok(_) => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret deleted"
            ),
            Err(KubeError::Api(ref err)) if err.code == 404 => {
                info!(
                    namespace = %self.namespace,
                    secret = %self.secret_name,
                    "secret already gone"
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
                "secret recreated (rollback)"
            ),
            Err(KubeError::Api(ref err)) if err.code == 409 => {
                info!(
                    namespace = %self.namespace,
                    secret = %self.secret_name,
                    "secret already exists (rollback)"
                );
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }
}
