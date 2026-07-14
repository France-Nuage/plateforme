pub use workflow_macros::OperationError;

pub mod execution;
pub mod fsm;
pub mod operations;
pub mod repository;
pub mod scheduler;
pub mod service;
pub mod workflows;

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

pub use frn_core::managed::PlatformConfig;
use frn_crypto::Kek;
use kube::Client as KubeClient;
use spicedb::SpiceDB;
use sqlx::PgPool;
use tokio::process::Command;

#[derive(Clone)]
pub struct WorkerContext {
    pub pool: PgPool,
    pub spicedb: SpiceDB,
    pub kube: KubeClient,
    pub platform_config: PlatformConfig,
    pub kek: Arc<Kek>,
    pub kubeconfig_path: Option<PathBuf>,
}

impl WorkerContext {
    /// Appends `--kubeconfig <path>` to a helm command when a target cluster
    /// kubeconfig has been resolved for the current execution.
    pub fn apply_kubeconfig(&self, command: &mut Command) {
        if let Some(path) = &self.kubeconfig_path {
            command.arg("--kubeconfig").arg(path);
        }
    }
}

impl fmt::Debug for WorkerContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerContext").finish_non_exhaustive()
    }
}
