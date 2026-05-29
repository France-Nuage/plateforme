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
    pub project_id: Uuid,
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

        let instance = self.find_instance(instance_id).await?;

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
        let platform_values = self.build_platform_values(&service.database_engine);
        let merged_values = merge_helm_values(user_vals, &platform_values);
        let secret_name = format!("{}-secrets", instance.release_name);
        let labels = build_instance_labels(&service.slug, instance.id, instance.project_id);

        scheduler
            .schedule(
                conn,
                DeleteManagedServiceParams {
                    instance_id: instance.id,
                    project_id: instance.project_id,
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
