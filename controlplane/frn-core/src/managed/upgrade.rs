use std::collections::BTreeMap;

use crate::authorization::{Authorize, Permission, Principal};
use crate::managed::{
    ManagedServiceError, ManagedServiceInstance, ManagedServiceInstanceStatus, ManagedServices,
    merge_helm_values, secret_values_to_map, transition_instance_status,
};
use crate::workflow::WorkflowScheduler;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UpgradeInstanceRequest {
    pub instance_id: Uuid,
    pub version_id: Uuid,
    pub user_values: Option<Value>,
    pub secret_values: Option<Value>,
}

#[derive(Debug)]
pub struct UpgradeManagedServiceParams {
    pub instance_id: Uuid,
    pub version_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub oci_reference: String,
    pub chart_version: String,
    pub merged_values: Value,
    pub secret_data: BTreeMap<String, String>,
}

impl<A: Authorize> ManagedServices<A> {
    pub async fn upgrade_instance<
        P: Principal + Sync,
        S: WorkflowScheduler<UpgradeManagedServiceParams>,
    >(
        &mut self,
        principal: &P,
        conn: &mut PgConnection,
        scheduler: &S,
        request: UpgradeInstanceRequest,
    ) -> Result<(), ManagedServiceError> {
        self.auth
            .can(principal)
            .perform(Permission::Update)
            .over::<ManagedServiceInstance>(&request.instance_id)
            .await?;

        let instance = self.find_instance(request.instance_id).await?;

        transition_instance_status(
            conn,
            instance.id,
            instance.status,
            ManagedServiceInstanceStatus::Upgrading,
        )
        .await?;

        let version = self.find_version_by_id(request.version_id).await?;

        let empty_obj = Value::Object(serde_json::Map::new());
        let user_vals = request
            .user_values
            .as_ref()
            .or(instance.user_values.as_ref())
            .unwrap_or(&empty_obj);
        let service = self.find_service_by_id(instance.service_id).await?;
        let platform_values = self.build_platform_values(&service.database_engine);
        let merged_values = merge_helm_values(user_vals, &platform_values);
        let secret_data = secret_values_to_map(&request.secret_values);
        let secret_name = format!("{}-secrets", instance.release_name);

        scheduler
            .schedule(
                conn,
                UpgradeManagedServiceParams {
                    instance_id: instance.id,
                    version_id: request.version_id,
                    namespace: instance.namespace.clone(),
                    release_name: instance.release_name.clone(),
                    secret_name,
                    oci_reference: version.oci_reference,
                    chart_version: version.chart_version,
                    merged_values,
                    secret_data,
                },
            )
            .await
            .map_err(ManagedServiceError::Workflow)?;

        Ok(())
    }
}
