use std::error::Error;
use std::sync::Arc;

use crate::WorkerContext;
use crate::execution::WorkflowExecutionId;

pub mod assert_namespace_absent;
pub mod check_permission;
pub mod create_k8s_secret;
pub mod create_namespace;
pub mod delete_k8s_secret;
pub mod delete_namespace;
pub mod delete_relationships;
pub mod helm_common;
pub mod helm_install;
pub mod helm_uninstall;
pub mod helm_upgrade;
pub mod k8s_common;
pub mod update_instance_status;
pub mod update_instance_version;
pub mod update_k8s_secret;
pub mod write_relationships;

pub trait OperationError: Error + Send + Sync {
    fn consume_retry(&self) -> bool;
    fn is_violated_invariant(&self) -> bool;
}

pub trait Operation: Sized + Clone + Send {
    type Error: OperationError;

    fn execute(
        self,
        ctx: WorkerContext,
        execution_id: WorkflowExecutionId,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;

    fn rollback(
        self,
        ctx: WorkerContext,
        execution_id: WorkflowExecutionId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn commit(
        self,
        _ctx: WorkerContext,
        _execution_id: WorkflowExecutionId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async { Ok(()) }
    }
}

impl OperationError for Arc<dyn OperationError + Send + Sync> {
    fn consume_retry(&self) -> bool {
        self.as_ref().consume_retry()
    }

    fn is_violated_invariant(&self) -> bool {
        self.as_ref().is_violated_invariant()
    }
}

#[macro_export]
macro_rules! operation_enum {
    ($($operation_name:ident,)*) => {
        paste::paste! {
            #[derive(Debug, Clone)]
            #[allow(clippy::large_enum_variant)]
            pub enum Operations {
                $($operation_name([<$operation_name Op>]),)*
            }

            impl $crate::operations::Operation for Operations {
                type Error = std::sync::Arc<dyn $crate::operations::OperationError + Send + Sync>;

                async fn execute(self, ctx: $crate::WorkerContext, execution_id: $crate::execution::WorkflowExecutionId) -> Result<Self, Self::Error> {
                    match self {
                        $(Self::$operation_name(op) => op.execute(ctx, execution_id).await.map_err(|e| std::sync::Arc::new(e) as Self::Error).map(Self::$operation_name)),*
                    }
                }

                async fn rollback(self, ctx: $crate::WorkerContext, execution_id: $crate::execution::WorkflowExecutionId) -> Result<(), Self::Error> {
                    match self {
                        $(Self::$operation_name(op) => op.rollback(ctx, execution_id).await.map_err(|e| std::sync::Arc::new(e) as Self::Error)),*
                    }
                }

                async fn commit(self, ctx: $crate::WorkerContext, execution_id: $crate::execution::WorkflowExecutionId) -> Result<(), Self::Error> {
                    match self {
                        $(Self::$operation_name(op) => op.commit(ctx, execution_id).await.map_err(|e| std::sync::Arc::new(e) as Self::Error)),*
                    }
                }
            }
        }
    };
}

use assert_namespace_absent::AssertNamespaceAbsentOp;
use check_permission::CheckPermissionOp;
use create_k8s_secret::CreateK8sSecretOp;
use create_namespace::CreateNamespaceOp;
use delete_k8s_secret::DeleteK8sSecretOp;
use delete_namespace::DeleteNamespaceOp;
use delete_relationships::DeleteRelationshipsOp;
use helm_install::HelmInstallOp;
use helm_uninstall::HelmUninstallOp;
use helm_upgrade::HelmUpgradeOp;
use update_instance_status::UpdateInstanceStatusOp;
use update_instance_version::UpdateInstanceVersionOp;
use update_k8s_secret::UpdateK8sSecretOp;
use write_relationships::WriteRelationshipsOp;

operation_enum! {
    AssertNamespaceAbsent,
    CheckPermission,
    CreateK8sSecret,
    CreateNamespace,
    DeleteK8sSecret,
    DeleteNamespace,
    DeleteRelationships,
    HelmInstall,
    HelmUninstall,
    HelmUpgrade,
    UpdateInstanceStatus,
    UpdateInstanceVersion,
    UpdateK8sSecret,
    WriteRelationships,
}

#[cfg(test)]
pub mod tests {
    use std::time::Duration;

    use thiserror::Error;
    use tokio::time::sleep;

    use crate::WorkerContext;
    use crate::execution::WorkflowExecutionId;
    use crate::operations::{Operation, OperationError};

    #[derive(Debug, Error)]
    pub enum ExecuteError<E: OperationError> {
        #[error("{0}")]
        OperationError(E),
        #[error("operation timed out")]
        TimeoutError,
    }

    pub async fn execute_operation<Op: Operation>(
        ctx: WorkerContext,
        operation: Op,
        execution_id: WorkflowExecutionId,
    ) -> Result<Op, ExecuteError<Op::Error>> {
        let mut tries = 0;
        loop {
            if tries >= 10 {
                return Err(ExecuteError::TimeoutError);
            }

            match operation.clone().execute(ctx.clone(), execution_id).await {
                Ok(op) => return Ok(op),
                Err(e) if e.consume_retry() => return Err(ExecuteError::OperationError(e)),
                Err(_) => sleep(Duration::from_millis(500 * tries)).await,
            }

            tries += 1;
        }
    }
}

/// Verifies the semantics the `#[derive(OperationError)]` macro attaches to
/// each `#[operation_error(...)]` marker, exercised through the real operation
/// error enums that carry those markers.
#[cfg(test)]
mod classification {
    use std::io::Error as IoError;

    use crate::operations::OperationError;
    use crate::operations::helm_install::HelmInstallError;
    use crate::operations::update_instance_status::UpdateInstanceStatusError;

    fn serde_error() -> serde_json::Error {
        serde_json::from_str::<i32>("not-an-int").unwrap_err()
    }

    #[test]
    fn invariant_variant_is_flagged_as_violated_invariant() {
        let error = UpdateInstanceStatusError::InvalidTransition("bad".to_owned());

        assert!(error.is_violated_invariant());
    }

    #[test]
    fn invariant_variant_still_consumes_a_retry() {
        let error = UpdateInstanceStatusError::InvalidTransition("bad".to_owned());

        assert!(error.consume_retry());
    }

    #[test]
    fn transient_variant_does_not_consume_a_retry() {
        let error = UpdateInstanceStatusError::FsmDatabase(sqlx::Error::RowNotFound);

        assert!(!error.consume_retry());
    }

    #[test]
    fn transient_variant_is_not_an_invariant() {
        let error = UpdateInstanceStatusError::FsmDatabase(sqlx::Error::RowNotFound);

        assert!(!error.is_violated_invariant());
    }

    #[test]
    fn unmarked_variant_consumes_a_retry() {
        let error = HelmInstallError::Failed("helm exited 1".to_owned());

        assert!(error.consume_retry());
    }

    #[test]
    fn unmarked_variant_is_not_an_invariant() {
        let error = HelmInstallError::Failed("helm exited 1".to_owned());

        assert!(!error.is_violated_invariant());
    }

    #[test]
    fn marked_variants_coexist_within_one_enum() {
        let transient = HelmInstallError::Io(IoError::other("boom"));
        let invariant = HelmInstallError::Serialization(serde_error());

        assert!(!transient.consume_retry() && invariant.is_violated_invariant());
    }
}
