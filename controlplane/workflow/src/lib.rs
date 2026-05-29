pub use workflow_macros::OperationError;

pub mod execution;
pub mod fsm;
pub mod operations;
pub mod repository;
pub mod scheduler;
pub mod service;
pub mod workflows;

use std::fmt;

pub use frn_core::managed::PlatformConfig;
use kube::Client as KubeClient;
use spicedb::SpiceDB;
use sqlx::PgPool;

#[derive(Clone)]
pub struct WorkerContext {
    pub pool: PgPool,
    pub spicedb: SpiceDB,
    pub kube: KubeClient,
    pub platform_config: PlatformConfig,
}

impl fmt::Debug for WorkerContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerContext").finish_non_exhaustive()
    }
}
