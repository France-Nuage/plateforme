use std::io::Error as IoError;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

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
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let values_json = serde_json::to_vec(&self.values)?;

        let mut child = Command::new("helm")
            .args([
                "install",
                &self.release_name,
                &self.chart_reference,
                "--version",
                &self.chart_version,
                "--namespace",
                &self.namespace,
                "--values",
                "-",
                "--wait",
                "--timeout",
                "5m0s",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("stdin was configured as piped");
        stdin.write_all(&values_json).await?;
        drop(stdin);

        let output = child.wait_with_output().await?;

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
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        let output = Command::new("helm")
            .args([
                "uninstall",
                &self.release_name,
                "--namespace",
                &self.namespace,
            ])
            .output()
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
