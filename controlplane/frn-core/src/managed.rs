//! Managed services catalog and versioning.
//!
//! Provides entity definitions and the service layer for managing
//! the marketplace of managed services (Vaultwarden, Nextcloud, etc.).

mod create;
mod delete;
mod seed;
mod upgrade;

pub use create::*;
pub use delete::*;
pub use seed::*;
pub use upgrade::*;

use std::collections::BTreeMap;

use crate::authorization::{Authorize, Resource};
use crate::kubernetes::KubernetesClusters;
use crate::resourcemanager::{Organization, Project};
use chrono::{DateTime, Utc};
use fabrique::sql::operators::Direction;
use fabrique::{Factory, Model, Query};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgConnection, Pool, Postgres};
use strum_macros::{Display, EnumString};
use thiserror::Error;
use uuid::Uuid;

#[derive(
    Debug, Clone, Dummy, Serialize, Deserialize, sqlx::Type, Display, EnumString, PartialEq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "managed_service_category", rename_all = "snake_case")]
pub enum ManagedServiceCategory {
    Security,
    Collaboration,
    Analytics,
    Database,
    Automation,
    Cms,
    Erp,
    Storage,
    Dashboard,
}

#[derive(
    Debug, Clone, Dummy, Serialize, Deserialize, sqlx::Type, Display, EnumString, PartialEq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "managed_database_engine", rename_all = "snake_case")]
pub enum ManagedDatabaseEngine {
    Cnpg,
    Mariadb,
}

#[derive(Debug, Clone, Factory, Model, Serialize)]
#[fabrique(table = "managed.service")]
pub struct ManagedService {
    #[fabrique(primary_key)]
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: ManagedServiceCategory,
    pub database_engine: Option<ManagedDatabaseEngine>,
    pub icon_url: Option<String>,
    /// Label selector resolved at instance deployment: a JSON object of
    /// key/value pairs (e.g. `{"availability": "ft"}`). Only healthy clusters
    /// carrying every pair are eligible to host instances of this service.
    /// `None` or `{}` means the service cannot be deployed
    /// ([`ManagedServiceError::MissingDeployTarget`]).
    pub deploy_target: Option<Value>,
    #[fabrique(soft_delete)]
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "managed.service_version")]
pub struct ManagedServiceVersion {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(belongs_to = ManagedService)]
    pub service_id: Uuid,
    pub chart_version: String,
    pub app_version: Option<String>,
    pub oci_reference: String,
    pub configurable_values_schema: Option<Value>,
    pub ui_schema: Option<Value>,
    #[fabrique(soft_delete)]
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A single entitlement entry within a plan (support level, GTI, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEntitlement {
    pub key: String,
    pub label: String,
    pub value: String,
}

/// A pricing tier for a managed service.
///
/// Each service can offer multiple plans with different Helm values,
/// entitlements (SLA guarantees), and pricing. Plans are synced from
/// the charts repository `catalogue.yaml` via the `SyncPlans` RPC.
#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "managed.service_plan")]
pub struct ManagedServicePlan {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(belongs_to = ManagedService)]
    pub service_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub highlighted: bool,
    pub values_override: Option<Value>,
    pub entitlements: Value,
    pub price_monthly_cents: Option<i64>,
    pub price_yearly_cents: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    sqlx::Type,
    Display,
    EnumString,
    Default,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ManagedServiceInstanceStatus {
    #[default]
    Provisioning,
    Running,
    Upgrading,
    Failed,
    Deleting,
    Deleted,
}

#[derive(Debug, Clone, Model, Resource, Serialize)]
#[fabrique(table = "managed.service_instance")]
pub struct ManagedServiceInstance {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(belongs_to = ManagedService)]
    pub service_id: Uuid,
    #[fabrique(belongs_to = ManagedServiceVersion)]
    pub version_id: Uuid,
    #[fabrique(belongs_to = ManagedServicePlan)]
    pub plan_id: Option<Uuid>,
    pub project_slug: String,
    pub organization_slug: String,
    /// The Kubernetes cluster hosting this instance, resolved at creation by
    /// matching the service `deploy_target` against the cluster labels.
    pub cluster_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub user_values: Option<Value>,
    /// References `lib_fsm.state_machine(state_machine__id)`.
    pub status: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Read-only view that resolves the FSM status UUID to an enum via the
/// `managed.service_instance_view` database view.
#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "managed.service_instance_view")]
pub struct ManagedServiceInstanceView {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(belongs_to = ManagedService)]
    pub service_id: Uuid,
    #[fabrique(belongs_to = ManagedServiceVersion)]
    pub version_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub project_slug: String,
    pub organization_slug: String,
    pub cluster_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub user_values: Option<Value>,
    pub status: ManagedServiceInstanceStatus,
    pub created_at: DateTime<Utc>,
}

/// Deep-merges user values with platform-generated values.
/// Priority: platform > user (platform overrides user).
pub fn merge_helm_values(user: &Value, platform: &Value) -> Value {
    let mut merged = user.clone();
    deep_merge(&mut merged, platform);
    merged
}

fn deep_merge(base: &mut Value, overlay: &Value) {
    if base.is_object() && overlay.is_object() {
        let base_map = base.as_object_mut().unwrap();
        let overlay_map = overlay.as_object().unwrap();
        for (key, value) in overlay_map {
            deep_merge(base_map.entry(key.clone()).or_insert(Value::Null), value);
        }
    } else {
        *base = overlay.clone();
    }
}

#[derive(Debug, Error)]
pub enum ManagedServiceError {
    #[error("authorization error: {0}")]
    Authorization(crate::Error),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("fabrique error: {0}")]
    Fabrique(#[from] fabrique::Error),
    #[error("service not found: {0}")]
    ServiceNotFound(String),
    #[error("version not found: {0}")]
    VersionNotFound(String),
    #[error("instance not found: {0}")]
    InstanceNotFound(Uuid),
    #[error("organization not found: {0}")]
    OrganizationNotFound(String),
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("service {0} declares no deploy_target and cannot be deployed")]
    MissingDeployTarget(String),
    #[error("service {0} has an invalid deploy_target: {1}")]
    InvalidDeployTarget(String, String),
    #[error("no healthy cluster matches the deploy_target of service {0}")]
    NoClusterMatchingDeployTarget(String),
    #[error("version already exists: {0}")]
    VersionAlreadyExists(String),
    #[error("invalid operation on instance {0}: current status is {1}")]
    InvalidInstanceStatus(Uuid, ManagedServiceInstanceStatus),
    #[error("namespace too long ({max_length} chars max): {namespace}")]
    NamespaceTooLong {
        namespace: String,
        max_length: usize,
    },
    #[error("plan not found: {0}")]
    PlanNotFound(String),
    #[error("plan is not active: {0}")]
    PlanNotActive(String),
    #[error("plan {plan_id} does not belong to service {service_id}")]
    PlanServiceMismatch { plan_id: Uuid, service_id: Uuid },
    #[error("workflow scheduling error: {0}")]
    Workflow(String),
}

impl From<crate::Error> for ManagedServiceError {
    fn from(err: crate::Error) -> Self {
        ManagedServiceError::Authorization(err)
    }
}

/// Transitions the FSM of a managed service instance within the given
/// transaction. Returns `InvalidInstanceStatus` when no valid transition
/// exists from the current state to `target_status`.
pub(crate) async fn transition_instance_status(
    conn: &mut PgConnection,
    instance_id: Uuid,
    state_machine_id: Uuid,
    target_status: ManagedServiceInstanceStatus,
) -> Result<(), ManagedServiceError> {
    let target_name = target_status.to_string();

    let event = sqlx::query_scalar::<_, String>(
        r#"SELECT abt.event
           FROM lib_fsm.state_machine sm
           JOIN lib_fsm.abstract_transition abt
               ON abt.from_abstract_state__id = sm.abstract_state__id
           JOIN lib_fsm.abstract_state target
               ON target.abstract_state__id = abt.to_abstract_state__id
           WHERE sm.state_machine__id = $1
             AND target.name = $2"#,
    )
    .bind(state_machine_id)
    .bind(&target_name)
    .fetch_optional(&mut *conn)
    .await?;

    let Some(event_name) = event else {
        let current_name = sqlx::query_scalar::<_, String>(
            r#"SELECT abs.name
               FROM lib_fsm.state_machine sm
               INNER JOIN lib_fsm.abstract_state abs
                   ON abs.abstract_state__id = sm.abstract_state__id
               WHERE sm.state_machine__id = $1"#,
        )
        .bind(state_machine_id)
        .fetch_one(&mut *conn)
        .await?;

        let current_status = current_name.parse().unwrap_or_default();
        return Err(ManagedServiceError::InvalidInstanceStatus(
            instance_id,
            current_status,
        ));
    };

    sqlx::query(r#"SELECT lib_fsm.state_machine_transition($1, $2)"#)
        .bind(state_machine_id)
        .bind(event_name)
        .execute(conn)
        .await?;

    Ok(())
}

const DEFAULT_ENVIRONMENT: &str = "prod";
const MAX_NAMESPACE_LENGTH: usize = 63;

pub fn generate_namespace(
    organization_slug: &str,
    service_slug: &str,
    instance_number: i64,
) -> Result<String, ManagedServiceError> {
    let namespace = if instance_number <= 1 {
        format!("managed-{organization_slug}-{service_slug}-{DEFAULT_ENVIRONMENT}")
    } else {
        format!(
            "managed-{organization_slug}-{service_slug}-{instance_number}-{DEFAULT_ENVIRONMENT}"
        )
    };
    if namespace.len() > MAX_NAMESPACE_LENGTH {
        return Err(ManagedServiceError::NamespaceTooLong {
            namespace,
            max_length: MAX_NAMESPACE_LENGTH,
        });
    }
    Ok(namespace)
}

pub fn generate_release_name(service_slug: &str, instance_number: i64) -> String {
    if instance_number <= 1 {
        service_slug.to_owned()
    } else {
        format!("{service_slug}-{instance_number}")
    }
}

/// Parses a service `deploy_target` JSON object into the label pairs a
/// hosting cluster must carry. Rejects a missing or empty target and any
/// non-string value with a typed error.
pub(crate) fn parse_deploy_target(
    service: &ManagedService,
) -> Result<BTreeMap<String, String>, ManagedServiceError> {
    let slug = &service.slug;
    let target = service
        .deploy_target
        .as_ref()
        .ok_or_else(|| ManagedServiceError::MissingDeployTarget(slug.clone()))?;

    let object = target.as_object().ok_or_else(|| {
        ManagedServiceError::InvalidDeployTarget(slug.clone(), "expected a JSON object".to_owned())
    })?;

    if object.is_empty() {
        return Err(ManagedServiceError::MissingDeployTarget(slug.clone()));
    }

    object
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|v| (key.clone(), v.to_owned()))
                .ok_or_else(|| {
                    ManagedServiceError::InvalidDeployTarget(
                        slug.clone(),
                        format!("value of '{key}' must be a string"),
                    )
                })
        })
        .collect()
}

pub fn build_instance_labels(
    service_slug: &str,
    instance_id: Uuid,
    project_slug: String,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/managed-by".to_owned(),
            "france-nuage".to_owned(),
        ),
        ("france-nuage/service".to_owned(), service_slug.to_owned()),
        ("france-nuage/instance".to_owned(), instance_id.to_string()),
        ("france-nuage/project".to_owned(), project_slug),
    ])
}

#[derive(Clone)]
pub struct PlatformConfig {
    pub default_storage_class: Option<String>,
}

#[derive(Clone)]
pub struct ManagedServices<A: Authorize> {
    pub(crate) auth: A,
    pub(crate) db: Pool<Postgres>,
    pub(crate) platform_config: PlatformConfig,
}

impl<A: Authorize> ManagedServices<A> {
    pub fn new(auth: A, db: Pool<Postgres>, platform_config: PlatformConfig) -> Self {
        Self {
            auth,
            db,
            platform_config,
        }
    }

    pub(crate) fn build_platform_values(
        &self,
        database_engine: &Option<ManagedDatabaseEngine>,
    ) -> Value {
        let mut map = serde_json::Map::new();

        if let Some(sc) = &self.platform_config.default_storage_class {
            let mut persistence = serde_json::Map::new();
            persistence.insert("storageClass".to_owned(), Value::String(sc.clone()));
            map.insert("persistence".to_owned(), Value::Object(persistence.clone()));

            if database_engine.is_some() {
                let mut cnpg = serde_json::Map::new();
                cnpg.insert("storageClass".to_owned(), Value::String(sc.clone()));
                map.insert("cnpg".to_owned(), Value::Object(cnpg));
            }
        }

        Value::Object(map)
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, Postgres>, sqlx::Error> {
        self.db.begin().await
    }

    pub(crate) async fn find_organization(
        &self,
        organization_slug: &str,
    ) -> Result<Organization, ManagedServiceError> {
        Organization::query()
            .select()
            .r#where(Organization::SLUG, "=", organization_slug.to_owned())
            .first(&self.db)
            .await?
            .ok_or_else(|| ManagedServiceError::OrganizationNotFound(organization_slug.to_owned()))
    }

    pub(crate) async fn find_project(
        &self,
        project_slug: &str,
    ) -> Result<Project, ManagedServiceError> {
        Project::query()
            .select()
            .r#where(Project::SLUG, "=", project_slug.to_owned())
            .first(&self.db)
            .await?
            .ok_or_else(|| ManagedServiceError::ProjectNotFound(project_slug.to_owned()))
    }

    /// Resolves the cluster that will host a new instance of `service`.
    ///
    /// Parses the service `deploy_target` into required labels, then picks a
    /// random healthy cluster carrying all of them. Fails with a typed error
    /// when the service declares no target
    /// ([`ManagedServiceError::MissingDeployTarget`]) or when no cluster
    /// matches ([`ManagedServiceError::NoClusterMatchingDeployTarget`]).
    pub(crate) async fn resolve_deploy_cluster(
        &self,
        service: &ManagedService,
    ) -> Result<Uuid, ManagedServiceError> {
        let required_labels = parse_deploy_target(service)?;
        KubernetesClusters::pick_healthy_cluster_matching(&self.db, &required_labels)
            .await?
            .map(|cluster| cluster.id)
            .ok_or_else(|| ManagedServiceError::NoClusterMatchingDeployTarget(service.slug.clone()))
    }

    pub(crate) async fn count_instances_for_service(
        &self,
        project_slug: &str,
        service_id: Uuid,
    ) -> Result<i64, ManagedServiceError> {
        // Raw SQL: aggregate COUNT not supported by fabrique query builder
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) as \"count!\" FROM managed.service_instance \
             WHERE project_slug = $1::citext AND service_id = $2",
            project_slug,
            service_id,
        )
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }

    pub async fn list_services(&self) -> Result<Vec<ManagedService>, ManagedServiceError> {
        ManagedService::query()
            .select()
            .where_null(ManagedService::DEACTIVATED_AT)
            .order_by(ManagedService::NAME, Direction::Asc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn find_service_by_slug(
        &self,
        slug: &str,
    ) -> Result<ManagedService, ManagedServiceError> {
        ManagedService::query()
            .select()
            .r#where(ManagedService::SLUG, "=", slug.to_owned())
            .where_null(ManagedService::DEACTIVATED_AT)
            .first(&self.db)
            .await?
            .ok_or_else(|| ManagedServiceError::ServiceNotFound(slug.to_owned()))
    }

    pub async fn find_service_by_id(
        &self,
        service_id: Uuid,
    ) -> Result<ManagedService, ManagedServiceError> {
        ManagedService::query()
            .select()
            .r#where(ManagedService::ID, "=", service_id)
            .where_null(ManagedService::DEACTIVATED_AT)
            .first(&self.db)
            .await?
            .ok_or_else(|| ManagedServiceError::ServiceNotFound(service_id.to_string()))
    }

    pub async fn list_versions(
        &self,
        service_slug: &str,
    ) -> Result<Vec<ManagedServiceVersion>, ManagedServiceError> {
        let service = self.find_service_by_slug(service_slug).await?;
        ManagedServiceVersion::query()
            .select()
            .r#where(ManagedServiceVersion::SERVICE_ID, "=", service.id)
            .where_null(ManagedServiceVersion::DEACTIVATED_AT)
            .order_by(ManagedServiceVersion::CREATED_AT, Direction::Desc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn register_version(
        &self,
        conn: &mut PgConnection,
        service_slug: &str,
        chart_version: &str,
        app_version: Option<&str>,
        oci_reference: &str,
        configurable_values_schema: Option<&Value>,
        ui_schema: Option<&Value>,
    ) -> Result<ManagedServiceVersion, ManagedServiceError> {
        let service = ManagedService::query()
            .select()
            .r#where(ManagedService::SLUG, "=", service_slug.to_owned())
            .where_null(ManagedService::DEACTIVATED_AT)
            .first(&mut *conn)
            .await?
            .ok_or_else(|| ManagedServiceError::ServiceNotFound(service_slug.to_owned()))?;

        let version = sqlx::query_as::<_, ManagedServiceVersion>(
            r#"INSERT INTO managed.service_version
                   (id, service_id, chart_version, app_version, oci_reference, configurable_values_schema, ui_schema)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (service_id, chart_version) DO NOTHING
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(service.id)
        .bind(chart_version)
        .bind(app_version)
        .bind(oci_reference)
        .bind(configurable_values_schema)
        .bind(ui_schema)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| {
            ManagedServiceError::VersionAlreadyExists(format!("{}/{}", service_slug, chart_version))
        })?;

        Ok(version)
    }

    pub async fn list_instances_by_project(
        &self,
        project_slug: &str,
    ) -> Result<Vec<ManagedServiceInstanceView>, ManagedServiceError> {
        ManagedServiceInstanceView::query()
            .select()
            .r#where(
                ManagedServiceInstanceView::PROJECT_SLUG,
                "=",
                project_slug.to_owned(),
            )
            .order_by(ManagedServiceInstanceView::CREATED_AT, Direction::Desc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn find_instance_with_status(
        &self,
        instance_id: Uuid,
    ) -> Result<ManagedServiceInstanceView, ManagedServiceError> {
        ManagedServiceInstanceView::query()
            .select()
            .r#where(ManagedServiceInstanceView::ID, "=", instance_id)
            .first(&self.db)
            .await?
            .ok_or(ManagedServiceError::InstanceNotFound(instance_id))
    }

    pub async fn find_instance(
        &self,
        instance_id: Uuid,
    ) -> Result<ManagedServiceInstance, ManagedServiceError> {
        ManagedServiceInstance::query()
            .select()
            .r#where(ManagedServiceInstance::ID, "=", instance_id)
            .first(&self.db)
            .await?
            .ok_or(ManagedServiceError::InstanceNotFound(instance_id))
    }

    pub async fn find_version_by_id(
        &self,
        version_id: Uuid,
    ) -> Result<ManagedServiceVersion, ManagedServiceError> {
        ManagedServiceVersion::query()
            .select()
            .r#where(ManagedServiceVersion::ID, "=", version_id)
            .where_null(ManagedServiceVersion::DEACTIVATED_AT)
            .first(&self.db)
            .await?
            .ok_or_else(|| ManagedServiceError::VersionNotFound(version_id.to_string()))
    }

    pub async fn update_instance_version(
        &self,
        instance_id: Uuid,
        version_id: Uuid,
    ) -> Result<(), ManagedServiceError> {
        ManagedServiceInstance::query()
            .update()
            .set(ManagedServiceInstance::VERSION_ID, version_id)
            .r#where(ManagedServiceInstance::ID, "=", instance_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Lists active plans for a given service, ordered by creation date.
    pub async fn list_plans(
        &self,
        service_id: Uuid,
    ) -> Result<Vec<ManagedServicePlan>, ManagedServiceError> {
        ManagedServicePlan::query()
            .select()
            .r#where(ManagedServicePlan::SERVICE_ID, "=", service_id)
            .r#where(ManagedServicePlan::STATUS, "=", "active".to_owned())
            .order_by(ManagedServicePlan::CREATED_AT, Direction::Asc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Lists all plans (including archived) for a given service.
    pub async fn list_all_plans(
        &self,
        service_id: Uuid,
    ) -> Result<Vec<ManagedServicePlan>, ManagedServiceError> {
        ManagedServicePlan::query()
            .select()
            .r#where(ManagedServicePlan::SERVICE_ID, "=", service_id)
            .order_by(ManagedServicePlan::CREATED_AT, Direction::Asc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Finds a plan by its UUID.
    pub async fn find_plan_by_id(
        &self,
        plan_id: Uuid,
    ) -> Result<ManagedServicePlan, ManagedServiceError> {
        ManagedServicePlan::query()
            .select()
            .r#where(ManagedServicePlan::ID, "=", plan_id)
            .first(&self.db)
            .await?
            .ok_or_else(|| ManagedServiceError::PlanNotFound(plan_id.to_string()))
    }

    /// Upserts a plan for a service. Used by the `SyncPlans` RPC and the seed.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_plan(
        &self,
        conn: &mut PgConnection,
        service_id: Uuid,
        slug: &str,
        name: &str,
        description: Option<&str>,
        status: &str,
        highlighted: bool,
        values_override: Option<&Value>,
        entitlements: &Value,
        price_monthly_cents: Option<i64>,
        price_yearly_cents: Option<i64>,
    ) -> Result<ManagedServicePlan, ManagedServiceError> {
        let plan = sqlx::query_as::<_, ManagedServicePlan>(
            r#"INSERT INTO managed.service_plan
                   (id, service_id, slug, name, description, status, highlighted,
                    values_override, entitlements, price_monthly_cents, price_yearly_cents)
               VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               ON CONFLICT (service_id, slug) DO UPDATE SET
                   name = EXCLUDED.name,
                   description = EXCLUDED.description,
                   status = EXCLUDED.status,
                   highlighted = EXCLUDED.highlighted,
                   values_override = EXCLUDED.values_override,
                   entitlements = EXCLUDED.entitlements,
                   price_monthly_cents = EXCLUDED.price_monthly_cents,
                   price_yearly_cents = EXCLUDED.price_yearly_cents
               RETURNING *"#,
        )
        .bind(service_id)
        .bind(slug)
        .bind(name)
        .bind(description)
        .bind(status)
        .bind(highlighted)
        .bind(values_override)
        .bind(entitlements)
        .bind(price_monthly_cents)
        .bind(price_yearly_cents)
        .fetch_one(&mut *conn)
        .await?;
        Ok(plan)
    }
}
