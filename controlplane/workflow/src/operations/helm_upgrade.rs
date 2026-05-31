use std::io::Error as IoError;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::info;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;
use crate::operations::helm_common::{helm_run, helm_run_with_stdin};

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
                version = %self.chart_version,
                "helm release upgraded"
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // A retry after a worker crash can re-run an upgrade that already
            // succeeded: helm then reports the release as unchanged (or, when a
            // prior install never completed, as having no deployed revision).
            // Both are benign for an idempotent retry, so they must not consume
            // a hard retry the way a genuine failure does.
            if stderr.contains("no changes since last release")
                || stderr.contains("has no deployed releases")
            {
                info!(release = %self.release_name, "helm release already up to date");
            } else {
                return Err(HelmUpgradeError::Failed(stderr.into_owned()));
            }
        }

        Ok(self)
    }

    async fn rollback(
        self,
        ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> Result<(), Self::Error> {
        // Helm's built-in rollback reverts to the previous revision.
        let output = helm_run(
            &ctx,
            &[
                "rollback",
                self.release_name.as_str(),
                "0",
                "--namespace",
                self.namespace.as_str(),
                "--wait",
                "--timeout",
                "5m0s",
            ],
        )
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
