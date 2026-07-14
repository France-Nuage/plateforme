use std::collections::BTreeMap;

use crate::authorization::{Authorize, Permission, Principal};
use crate::managed::{
    ManagedServiceError, ManagedServiceInstance, ManagedServices, build_instance_labels,
    generate_namespace, generate_release_name, merge_helm_values,
};
use crate::resourcemanager::Project;
use crate::workflow::WorkflowScheduler;
use fabrique::Query;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateInstanceRequest {
    pub project_slug: String,
    pub organization_slug: String,
    pub service_slug: String,
    pub version_id: Uuid,
    pub plan_id: Uuid,
    pub user_values: Option<Value>,
    pub secret_values: Option<Value>,
}

/// Principal identity carried by a workflow for defense-in-depth permission
/// re-checks at execution time. Serializable so it survives the DB round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPrincipal {
    pub principal_type: String,
    pub principal_id: String,
}

#[derive(Debug)]
pub struct DeployManagedServiceParams {
    pub instance_id: Uuid,
    pub project_slug: String,
    pub cluster_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub oci_reference: String,
    pub chart_version: String,
    pub merged_values: Value,
    pub secret_data: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub principal: Option<WorkflowPrincipal>,
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
            .over::<Project>(&request.project_slug)
            .await?;

        let plan = self.find_plan_by_id(request.plan_id).await?;
        if plan.requires_payment {
            return Err(ManagedServiceError::PlanRequiresPayment(plan.slug.clone()));
        }

        let workflow_principal = Some(WorkflowPrincipal {
            principal_type: principal.name().to_owned(),
            principal_id: principal.id().to_string(),
        });

        self.create_instance_internal(conn, scheduler, request, workflow_principal)
            .await
    }

    /// Creates a managed service instance without authorization checks.
    ///
    /// Used by the billing webhook handler where authorization was already
    /// verified at checkout session creation time.
    pub async fn create_instance_unchecked<S: WorkflowScheduler<DeployManagedServiceParams>>(
        &self,
        conn: &mut PgConnection,
        scheduler: &S,
        request: CreateInstanceRequest,
    ) -> Result<ManagedServiceInstance, ManagedServiceError> {
        self.create_instance_internal(conn, scheduler, request, None)
            .await
    }

    async fn create_instance_internal<S: WorkflowScheduler<DeployManagedServiceParams>>(
        &self,
        conn: &mut PgConnection,
        scheduler: &S,
        request: CreateInstanceRequest,
        principal: Option<WorkflowPrincipal>,
    ) -> Result<ManagedServiceInstance, ManagedServiceError> {
        let organization = self.find_organization(&request.organization_slug).await?;
        self.find_project(&request.project_slug).await?;
        let service = self.find_service_by_slug(&request.service_slug).await?;

        // The hosting cluster is resolved here, at deployment time, by
        // matching the service deploy_target against the cluster labels (not
        // inherited from the project): each instance can land on a different
        // cluster as the fleet evolves.
        let cluster_id = self.resolve_deploy_cluster(&service).await?;

        let plan = self.find_plan_by_id(request.plan_id).await?;
        if plan.service_id != service.id {
            return Err(ManagedServiceError::PlanServiceMismatch {
                plan_id: plan.id,
                service_id: service.id,
            });
        }
        if plan.status != "active" {
            return Err(ManagedServiceError::PlanNotActive(plan.slug.clone()));
        }

        let versions = self.list_versions(&service.slug).await?;
        let version = versions
            .iter()
            .find(|v| v.id == request.version_id)
            .ok_or_else(|| ManagedServiceError::VersionNotFound(request.version_id.to_string()))?;

        let instance_id = Uuid::new_v4();
        let existing_count = self
            .count_instances_for_service(&request.project_slug, service.id)
            .await?;
        let instance_number = existing_count + 1;
        let namespace = generate_namespace(&organization.slug, &service.slug, instance_number)?;
        let release_name = generate_release_name(&service.slug, instance_number);
        let secret_name = format!("{}-secrets", service.slug);

        let empty_obj = Value::Object(serde_json::Map::new());
        let user_vals = request.user_values.as_ref().unwrap_or(&empty_obj);
        let plan_vals = plan.values_override.as_ref().unwrap_or(&empty_obj);
        let user_plus_plan = merge_helm_values(user_vals, plan_vals);
        let platform_values = self.build_platform_values(&service);
        let merged_values = merge_helm_values(&user_plus_plan, &platform_values);
        let secret_data = secret_values_to_map(&request.secret_values);
        let labels = build_instance_labels(&service.slug, instance_id, &request.project_slug);

        let instance = ManagedServiceInstance::query()
            .insert()
            .set(ManagedServiceInstance::ID, instance_id)
            .set(ManagedServiceInstance::SERVICE_ID, service.id)
            .set(ManagedServiceInstance::VERSION_ID, request.version_id)
            .set(
                ManagedServiceInstance::PLAN_ID,
                Some(request.plan_id) as Option<Uuid>,
            )
            .set(
                ManagedServiceInstance::PROJECT_SLUG,
                request.project_slug.clone(),
            )
            .set(
                ManagedServiceInstance::ORGANIZATION_SLUG,
                request.organization_slug,
            )
            .set(ManagedServiceInstance::CLUSTER_ID, cluster_id)
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
                    project_slug: request.project_slug,
                    cluster_id,
                    namespace,
                    release_name,
                    secret_name,
                    oci_reference: version.oci_reference.clone(),
                    chart_version: version.chart_version.clone(),
                    merged_values,
                    secret_data,
                    labels,
                    principal,
                },
            )
            .await
            .map_err(ManagedServiceError::Workflow)?;

        Ok(instance)
    }
}

pub fn secret_values_to_map(value: &Option<Value>) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    if let Some(obj) = value.as_ref().and_then(|v| v.as_object()) {
        flatten_secret_values("", obj, &mut result);
    }
    result
}

fn flatten_secret_values(
    prefix: &str,
    obj: &serde_json::Map<String, Value>,
    result: &mut BTreeMap<String, String>,
) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            Value::String(s) => {
                result.insert(full_key, s.clone());
            }
            Value::Object(nested) => {
                flatten_secret_values(&full_key, nested, result);
            }
            other => {
                result.insert(full_key, other.to_string());
            }
        }
    }
}
