use std::fmt::Debug;

use chrono::{DateTime, Utc};

use crate::WorkerContext;
use crate::operations::Operations;

pub mod delete_managed_service;
pub mod deploy_managed_service;
pub mod upgrade_managed_service;
pub mod write_relationships;

#[derive(Debug)]
pub struct ScheduledWorkflow {
    pub workflow: WorkflowDefinitions,
    pub schedule_at: Option<DateTime<Utc>>,
}

impl ScheduledWorkflow {
    pub fn immediate(workflow: WorkflowDefinitions) -> Self {
        Self {
            workflow,
            schedule_at: None,
        }
    }

    pub fn at(workflow: WorkflowDefinitions, schedule_at: DateTime<Utc>) -> Self {
        Self {
            workflow,
            schedule_at: Some(schedule_at),
        }
    }
}

type WorkflowListResult<E> = Result<Vec<WorkflowDefinitions>, E>;
type ScheduledWorkflowListResult<E> = Result<Vec<ScheduledWorkflow>, E>;
type OperationListResult<E> = Result<Vec<Operations>, E>;

pub trait WorkflowDefinition: Debug + Sized + Send {
    type Error: Debug;

    fn needed_workflows(
        &mut self,
        _ctx: WorkerContext,
    ) -> impl Future<Output = WorkflowListResult<Self::Error>> + Send {
        async { Ok(vec![]) }
    }

    fn next_operations(
        &mut self,
        ctx: WorkerContext,
    ) -> impl Future<Output = OperationListResult<Self::Error>> + Send;

    fn next_workflows(
        &self,
        _ctx: WorkerContext,
    ) -> impl Future<Output = ScheduledWorkflowListResult<Self::Error>> + Send {
        async { Ok(vec![]) }
    }

    fn name(&self) -> &str;
}

#[macro_export]
macro_rules! workflow_enum {
    ($($workflow_name:ident,)*) => {
        paste::paste! {
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub enum WorkflowDefinitions {
                $($workflow_name([<$workflow_name Workflow>]),)*
            }

            impl WorkflowDefinitions {
                pub fn name(&self) -> &str {
                    match self {
                        $(Self::$workflow_name(workflow) => workflow.name()),*
                    }
                }
            }

            impl $crate::workflows::WorkflowDefinition for WorkflowDefinitions {
                type Error = Box<dyn std::error::Error>;

                async fn needed_workflows(&mut self, ctx: $crate::WorkerContext) -> Result<Vec<WorkflowDefinitions>, Self::Error> {
                    match self {
                        $(Self::$workflow_name(workflow) => workflow.needed_workflows(ctx).await.map_err(|e| e.into())),*
                    }
                }

                async fn next_operations(&mut self, ctx: $crate::WorkerContext) -> Result<Vec<$crate::operations::Operations>, Self::Error> {
                    match self {
                        $(Self::$workflow_name(workflow) => workflow.next_operations(ctx).await.map_err(|e| e.into())),*
                    }
                }

                async fn next_workflows(&self, ctx: $crate::WorkerContext) -> Result<Vec<$crate::workflows::ScheduledWorkflow>, Self::Error> {
                    match self {
                        $(Self::$workflow_name(workflow) => workflow.next_workflows(ctx).await.map_err(|e| e.into())),*
                    }
                }

                fn name(&self) -> &str {
                    match self {
                        $(Self::$workflow_name(workflow) => workflow.name()),*
                    }
                }
            }
        }
    };
}

use delete_managed_service::DeleteManagedServiceWorkflow;
use deploy_managed_service::DeployManagedServiceWorkflow;
use upgrade_managed_service::UpgradeManagedServiceWorkflow;
use write_relationships::WriteRelationshipsWorkflow;

workflow_enum! {
    DeleteManagedService,
    DeployManagedService,
    UpgradeManagedService,
    WriteRelationships,
}
