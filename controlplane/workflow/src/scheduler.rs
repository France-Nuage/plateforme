use frn_core::managed::{
    DeleteManagedServiceParams, DeployManagedServiceParams, UpgradeManagedServiceParams,
};
use frn_core::workflow::WorkflowScheduler;
use sqlx::PgConnection;

use crate::execution::WorkflowInitiator;
use crate::service::WorkflowService;
use crate::workflows::WorkflowDefinitions;
use crate::workflows::delete_managed_service::DeleteManagedServiceWorkflow;
use crate::workflows::deploy_managed_service::DeployManagedServiceWorkflow;
use crate::workflows::upgrade_managed_service::UpgradeManagedServiceWorkflow;

const WORKFLOW_MAX_RETRY: i32 = 3;

#[derive(Clone)]
pub struct ManagedWorkflowScheduler;

impl WorkflowScheduler<DeployManagedServiceParams> for ManagedWorkflowScheduler {
    async fn schedule(
        &self,
        conn: &mut PgConnection,
        params: DeployManagedServiceParams,
    ) -> Result<(), String> {
        WorkflowService::schedule_workflow(
            conn,
            WorkflowDefinitions::DeployManagedService(DeployManagedServiceWorkflow::new(
                params.instance_id,
                params.project_slug,
                params.cluster_id,
                params.namespace,
                params.release_name,
                params.secret_name,
                params.oci_reference,
                params.chart_version,
                params.merged_values,
                params.secret_data,
                params.labels,
                params.annotations,
                params.principal,
            )),
            WORKFLOW_MAX_RETRY,
            WorkflowInitiator::System,
            None,
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}

impl WorkflowScheduler<UpgradeManagedServiceParams> for ManagedWorkflowScheduler {
    async fn schedule(
        &self,
        conn: &mut PgConnection,
        params: UpgradeManagedServiceParams,
    ) -> Result<(), String> {
        WorkflowService::schedule_workflow(
            conn,
            WorkflowDefinitions::UpgradeManagedService(UpgradeManagedServiceWorkflow::new(
                params.instance_id,
                params.cluster_id,
                params.version_id,
                params.namespace,
                params.release_name,
                params.secret_name,
                params.oci_reference,
                params.chart_version,
                params.merged_values,
                params.secret_data,
            )),
            WORKFLOW_MAX_RETRY,
            WorkflowInitiator::System,
            None,
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}

impl WorkflowScheduler<DeleteManagedServiceParams> for ManagedWorkflowScheduler {
    async fn schedule(
        &self,
        conn: &mut PgConnection,
        params: DeleteManagedServiceParams,
    ) -> Result<(), String> {
        WorkflowService::schedule_workflow(
            conn,
            WorkflowDefinitions::DeleteManagedService(DeleteManagedServiceWorkflow::new(
                params.instance_id,
                params.project_slug,
                params.cluster_id,
                params.namespace,
                params.release_name,
                params.secret_name,
                params.oci_reference,
                params.chart_version,
                params.merged_values,
                params.labels,
            )),
            WORKFLOW_MAX_RETRY,
            WorkflowInitiator::System,
            None,
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}
