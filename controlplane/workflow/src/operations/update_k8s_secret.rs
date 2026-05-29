use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Secret;
use kube::Api;
use kube::Error as KubeError;
use kube::api::PostParams;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateK8sSecretOp {
    pub namespace: String,
    pub secret_name: String,
    pub data: BTreeMap<String, String>,
    /// Populated during execute to allow rollback to restore previous state.
    pub previous_data: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum UpdateK8sSecretError {
    #[error("kubernetes API error: {0}")]
    #[operation_error(transient)]
    Kube(#[from] KubeError),
}

impl crate::operations::Operation for UpdateK8sSecretOp {
    type Error = UpdateK8sSecretError;

    async fn execute(
        mut self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let secrets = Api::<Secret>::namespaced(ctx.kube, &self.namespace);

        let mut current = secrets.get(&self.secret_name).await?;

        self.previous_data = current.data.take().map(|d| {
            d.into_iter()
                .filter_map(|(k, v)| String::from_utf8(v.0).ok().map(|s| (k, s)))
                .collect()
        });

        current.string_data = Some(self.data.clone());

        secrets
            .replace(&self.secret_name, &PostParams::default(), &current)
            .await?;

        info!(
            namespace = %self.namespace,
            secret = %self.secret_name,
            "secret updated"
        );

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let Some(previous_data) = self.previous_data else {
            info!(
                namespace = %self.namespace,
                secret = %self.secret_name,
                "no previous data captured, skipping rollback"
            );
            return Ok(());
        };

        let secrets = Api::<Secret>::namespaced(ctx.kube, &self.namespace);

        let mut current = secrets.get(&self.secret_name).await?;
        current.data = None;
        current.string_data = Some(previous_data);

        secrets
            .replace(&self.secret_name, &PostParams::default(), &current)
            .await?;

        info!(
            namespace = %self.namespace,
            secret = %self.secret_name,
            "secret restored (rollback)"
        );

        Ok(())
    }
}
