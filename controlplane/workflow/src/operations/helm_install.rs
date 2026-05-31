use std::io::Error as IoError;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;
use crate::operations::helm_common::{helm_run, helm_run_with_stdin};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmInstallOp {
    pub release_name: String,
    pub namespace: String,
    pub chart_reference: String,
    pub chart_version: String,
    pub values: Value,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum HelmInstallError {
    #[error("failed to execute helm: {0}")]
    #[operation_error(transient)]
    Io(#[from] IoError),

    #[error("helm install failed: {0}")]
    Failed(String),

    #[error("failed to serialize values: {0}")]
    #[operation_error(invariant)]
    Serialization(#[from] serde_json::Error),
}

impl crate::operations::Operation for HelmInstallOp {
    type Error = HelmInstallError;

    async fn execute(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let values_json = serde_json::to_vec(&self.values)?;

        let output = helm_run_with_stdin(
            &ctx,
            &[
                "install",
                self.release_name.as_str(),
                self.chart_reference.as_str(),
                "--version",
                self.chart_version.as_str(),
                "--namespace",
                self.namespace.as_str(),
                "--values",
                "-",
                "--wait",
                "--timeout",
                "5m0s",
            ],
            &values_json,
        )
        .await?;

        if output.status.success() {
            info!(
                release = %self.release_name,
                namespace = %self.namespace,
                chart = %self.chart_reference,
                version = %self.chart_version,
                "helm release installed"
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("cannot re-use a name that is still in use") {
                info!(release = %self.release_name, "helm release already exists");
            } else {
                return Err(HelmInstallError::Failed(stderr.into_owned()));
            }
        }

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let output = helm_run(
            &ctx,
            &[
                "uninstall",
                self.release_name.as_str(),
                "--namespace",
                self.namespace.as_str(),
            ],
        )
        .await?;

        if output.status.success() {
            info!(release = %self.release_name, "helm release uninstalled (rollback)");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") {
                info!(release = %self.release_name, "helm release already gone (rollback)");
            } else {
                return Err(HelmInstallError::Failed(stderr.into_owned()));
            }
        }

        Ok(())
    }
}
