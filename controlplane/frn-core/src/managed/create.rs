use std::collections::BTreeMap;

use crate::authorization::{Authorize, Permission, Principal};
use crate::managed::{
    ManagedServiceError, ManagedServiceInstance, ManagedServices, build_instance_labels,
    generate_namespace, generate_release_name, merge_helm_values,
};
use crate::resourcemanager::Project;
use crate::workflow::WorkflowScheduler;
use fabrique::Query;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateInstanceRequest {
    pub project_id: Uuid,
    pub organization_id: Uuid,
    pub service_slug: String,
    pub version_id: Uuid,
    pub user_values: Option<Value>,
    pub secret_values: Option<Value>,
}

#[derive(Debug)]
pub struct DeployManagedServiceParams {
    pub instance_id: Uuid,
    pub project_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub oci_reference: String,
    pub chart_version: String,
    pub merged_values: Value,
    pub secret_data: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
}

impl<A: Authorize> ManagedServices<A> {
    pub async fn create_instance<
        P: Principal + Sync,
        S: WorkflowScheduler<DeployManagedServiceParams>,
    >(
        &mut self,
        principal: &P,
        conn: &mut PgConnection,
        scheduler: &S,
        request: CreateInstanceRequest,
    ) -> Result<ManagedServiceInstance, ManagedServiceError> {
        self.auth
            .can(principal)
            .perform(Permission::CreateInstance)
            .over::<Project>(&request.project_id)
            .await?;

        let organization = self.find_organization(request.organization_id).await?;
        let _project = self.find_project(request.project_id).await?;
        let service = self.find_service_by_slug(&request.service_slug).await?;

        let versions = self.list_versions(&service.slug).await?;
        let version = versions
            .iter()
            .find(|v| v.id == request.version_id)
            .ok_or_else(|| ManagedServiceError::VersionNotFound(request.version_id.to_string()))?;

        let instance_id = Uuid::now_v7();
        let existing_count = self
            .count_instances_for_service(request.project_id, service.id)
            .await?;
        let instance_number = existing_count + 1;
        let namespace = generate_namespace(&organization.slug, &service.slug, instance_number)?;
        let release_name = generate_release_name(&service.slug, instance_number);
        let secret_name = format!("{release_name}-secrets");

        let empty_obj = Value::Object(serde_json::Map::new());
        let user_vals = request.user_values.as_ref().unwrap_or(&empty_obj);
        let platform_values = self.build_platform_values(&service.database_engine);
        let merged_values = merge_helm_values(user_vals, &platform_values);
        let secret_data = secret_values_to_map(&request.secret_values);
        let labels = build_instance_labels(&service.slug, instance_id, request.project_id);

        let instance = ManagedServiceInstance::query()
            .insert()
            .set(ManagedServiceInstance::ID, instance_id)
            .set(ManagedServiceInstance::SERVICE_ID, service.id)
            .set(ManagedServiceInstance::VERSION_ID, request.version_id)
            .set(ManagedServiceInstance::PROJECT_ID, request.project_id)
            .set(
                ManagedServiceInstance::ORGANIZATION_ID,
                request.organization_id,
            )
            .set(ManagedServiceInstance::NAMESPACE, namespace.clone())
            .set(ManagedServiceInstance::RELEASE_NAME, release_name.clone())
            .set(ManagedServiceInstance::USER_VALUES, request.user_values)
            .returning()
            .first(&mut *conn)
            .await?
            .ok_or_else(|| ManagedServiceError::Database(sqlx::Error::RowNotFound))?;

        scheduler
            .schedule(
                conn,
                DeployManagedServiceParams {
                    instance_id: instance.id,
                    project_id: request.project_id,
                    namespace,
                    release_name,
                    secret_name,
                    oci_reference: version.oci_reference.clone(),
                    chart_version: version.chart_version.clone(),
                    merged_values,
                    secret_data,
                    labels,
                },
            )
            .await
            .map_err(ManagedServiceError::Workflow)?;

        Ok(instance)
    }
}

pub fn secret_values_to_map(value: &Option<Value>) -> BTreeMap<String, String> {
    value
        .as_ref()
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}
