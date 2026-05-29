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
pub struct HelmUpgradeOp {
    pub release_name: String,
    pub namespace: String,
    pub chart_reference: String,
    pub chart_version: String,
    pub values: Value,
}

#[derive(Debug, Error, crate::OperationError)]
pub enum HelmUpgradeError {
    #[error("failed to execute helm: {0}")]
    #[operation_error(transient)]
    Io(#[from] IoError),

    #[error("helm upgrade failed: {0}")]
    Failed(String),

    #[error("failed to serialize values: {0}")]
    #[operation_error(invariant)]
    Serialization(#[from] serde_json::Error),
}

impl crate::operations::Operation for HelmUpgradeOp {
    type Error = HelmUpgradeError;

    async fn execute(
        self,
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<Self, Self::Error> {
        let values_json = serde_json::to_vec(&self.values)?;

        let mut child = Command::new("helm")
            .args([
                "upgrade",
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
                version = %self.chart_version,
                "helm release upgraded"
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HelmUpgradeError::Failed(stderr.into_owned()));
        }

        Ok(self)
    }

    async fn rollback(
        self,
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        // Helm's built-in rollback reverts to the previous revision.
        let output = Command::new("helm")
            .args([
                "rollback",
                &self.release_name,
                "0",
                "--namespace",
                &self.namespace,
                "--wait",
                "--timeout",
                "5m0s",
            ])
            .output()
            .await?;

        if output.status.success() {
            info!(release = %self.release_name, "helm release rolled back");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HelmUpgradeError::Failed(stderr.into_owned()));
        }

        Ok(())
    }
}
