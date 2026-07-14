use std::collections::BTreeMap;

use crate::authorization::{Authorize, Permission, Principal};
use crate::managed::{
    ManagedServiceError, ManagedServiceInstance, ManagedServiceInstanceStatus, ManagedServices,
    build_instance_labels, merge_helm_values, transition_instance_status,
};
use crate::workflow::WorkflowScheduler;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug)]
pub struct DeleteManagedServiceParams {
    pub instance_id: Uuid,
    pub project_slug: String,
    pub cluster_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub oci_reference: String,
    pub chart_version: String,
    pub merged_values: Value,
    pub labels: BTreeMap<String, String>,
}

impl<A: Authorize> ManagedServices<A> {
    pub async fn delete_instance<
        P: Principal + Sync,
        S: WorkflowScheduler<DeleteManagedServiceParams>,
    >(
        &mut self,
        principal: &P,
        conn: &mut PgConnection,
        scheduler: &S,
        instance_id: Uuid,
    ) -> Result<(), ManagedServiceError> {
        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over::<ManagedServiceInstance>(&instance_id)
            .await?;

        // The instance stores the cluster its release lives on: the deletion
        // must target that exact cluster, not re-run the deploy_target
        // matching.
        let instance = self.find_instance(instance_id).await?;
        let cluster_id = instance.cluster_id;

        transition_instance_status(
            conn,
            instance.id,
            instance.status,
            ManagedServiceInstanceStatus::Deleting,
        )
        .await?;

        let service = self.find_service_by_id(instance.service_id).await?;
        let version = self.find_version_by_id(instance.version_id).await?;

        let empty_obj = Value::Object(serde_json::Map::new());
        let user_vals = instance.user_values.as_ref().unwrap_or(&empty_obj);
        let plan_vals = if let Some(plan_id) = instance.plan_id {
            let plan = self.find_plan_by_id(plan_id).await?;
            plan.values_override.unwrap_or_else(|| empty_obj.clone())
        } else {
            empty_obj.clone()
        };
        let user_plus_plan = merge_helm_values(user_vals, &plan_vals);
        let platform_values = self.build_platform_values(&service);
        let merged_values = merge_helm_values(&user_plus_plan, &platform_values);
        let secret_name = format!("{}-secrets", service.slug);
        let labels = build_instance_labels(&service.slug, instance.id, &instance.project_slug);

        scheduler
            .schedule(
                conn,
                DeleteManagedServiceParams {
                    instance_id: instance.id,
                    project_slug: instance.project_slug,
                    cluster_id,
                    namespace: instance.namespace.clone(),
                    release_name: instance.release_name.clone(),
                    secret_name,
                    oci_reference: version.oci_reference,
                    chart_version: version.chart_version,
                    merged_values,
                    labels,
                },
            )
            .await
            .map_err(ManagedServiceError::Workflow)?;

        Ok(())
    }
}
