use std::collections::BTreeMap;

use k8s_openapi::ByteString;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::{Api, Error as KubeError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;
use crate::operations::k8s_common::{
    CreateOutcome, DeleteOutcome, classify_create, classify_delete, classify_get,
};

/// Secret data is captured from K8s during execute to allow rollback.
/// Values are stored as raw bytes (base64-encoded by serde) to preserve
/// binary entries such as TLS certificates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteK8sSecretOp {
    pub namespace: String,
    pub secret_name: String,
    pub data: BTreeMap<String, Vec<u8>>,
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

        if let Some(secret) = classify_get(secrets.get(&self.secret_name).await)?
            && let Some(data) = secret.data
        {
            self.data = data.into_iter().map(|(k, v)| (k, v.0)).collect();
        }

        match classify_delete(secrets.delete(&self.secret_name, &Default::default()).await)? {
            DeleteOutcome::Deleted => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret deleted"
            ),
            DeleteOutcome::AlreadyGone => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret already gone"
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

        let data: BTreeMap<String, ByteString> = self
            .data
            .iter()
            .map(|(k, v)| (k.clone(), ByteString(v.clone())))
            .collect();

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(self.secret_name.clone()),
                namespace: Some(self.namespace.clone()),
                ..Default::default()
            },
            data: Some(data),
            ..Default::default()
        };

        match classify_create(secrets.create(&PostParams::default(), &secret).await)? {
            CreateOutcome::Created => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret recreated (rollback)"
            ),
            CreateOutcome::AlreadyExists => info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "secret already exists (rollback)"
            ),
        }

        Ok(())
    }
}
