use std::io::Error as IoError;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;
use crate::operations::helm_common::{
    HelmOutcome, classify_helm_result, helm_run, helm_run_with_stdin,
};

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
                "upgrade",
                self.release_name.as_str(),
                self.chart_reference.as_str(),
                "--install",
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

        match classify_helm_result(
            output.status.success(),
            &output.stderr,
            &["no changes since last release"],
        )
        .map_err(HelmInstallError::Failed)?
        {
            HelmOutcome::Applied => info!(
                release = %self.release_name,
                namespace = %self.namespace,
                chart = %self.chart_reference,
                version = %self.chart_version,
                "helm release installed"
            ),
            HelmOutcome::AlreadyReconciled => {
                info!(release = %self.release_name, "helm release already up to date")
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

        match classify_helm_result(output.status.success(), &output.stderr, &["not found"])
            .map_err(HelmInstallError::Failed)?
        {
            HelmOutcome::Applied => {
                info!(release = %self.release_name, "helm release uninstalled (rollback)")
            }
            HelmOutcome::AlreadyReconciled => {
                info!(release = %self.release_name, "helm release already gone (rollback)")
            }
        }

        Ok(())
    }
}
