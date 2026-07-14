use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::auth::authenticate_bearer;
use crate::error::Error;
use crate::timestamp::to_timestamp;
use frn_core::authorization::Authorize;
use frn_core::identity::IAM;
use frn_core::managed::{
    ManagedService, ManagedServiceError, ManagedServiceInstanceView, ManagedServicePlan,
    ManagedServiceVersion, ManagedServices, PlanEntitlement,
};
use frn_crypto::Kek;
use workflow::scheduler::ManagedWorkflowScheduler;

tonic::include_proto!("francenuage.fr.v1.managed");

pub struct ManagedServicesRpc<A: Authorize> {
    iam: IAM,
    service: ManagedServices<A>,
    pool: Pool<Postgres>,
    ci_token: String,
    kek: Arc<Kek>,
}

impl<A: Authorize> ManagedServicesRpc<A> {
    pub fn new(
        iam: IAM,
        service: ManagedServices<A>,
        pool: Pool<Postgres>,
        ci_token: String,
        kek: Arc<Kek>,
    ) -> Self {
        Self {
            iam,
            service,
            pool,
            ci_token,
            kek,
        }
    }

    fn authenticate_ci(&self, request: &Request<impl Sized>) -> Result<(), Status> {
        authenticate_bearer(request, &self.ci_token, "invalid CI service token")
    }
}

impl From<&ManagedService> for ManagedServiceProto {
    fn from(service: &ManagedService) -> Self {
        Self {
            id: service.id.to_string(),
            slug: service.slug.clone(),
            name: service.name.clone(),
            description: service.description.clone(),
            category: service.category.to_string(),
            database_engine: service.database_engine.as_ref().map(|e| e.to_string()),
            icon_url: service.icon_url.clone(),
            created_at: Some(to_timestamp(service.created_at)),
        }
    }
}

impl From<&ManagedServiceVersion> for ManagedServiceVersionProto {
    fn from(version: &ManagedServiceVersion) -> Self {
        Self {
            id: version.id.to_string(),
            service_id: version.service_id.to_string(),
            chart_version: version.chart_version.clone(),
            app_version: version.app_version.clone(),
            oci_reference: version.oci_reference.clone(),
            configurable_values_schema: version
                .configurable_values_schema
                .as_ref()
                .map(|s| s.to_string()),
            created_at: Some(to_timestamp(version.created_at)),
            ui_schema: version.ui_schema.as_ref().map(|s| s.to_string()),
            connection_info_schema: version
                .connection_info_schema
                .as_ref()
                .map(|s| s.to_string()),
        }
    }
}

impl From<&ManagedServiceInstanceView> for ManagedServiceInstanceProto {
    fn from(i: &ManagedServiceInstanceView) -> Self {
        Self {
            id: i.id.to_string(),
            service_id: i.service_id.to_string(),
            version_id: i.version_id.to_string(),
            project_slug: i.project_slug.clone(),
            organization_slug: i.organization_slug.clone(),
            namespace: i.namespace.clone(),
            release_name: i.release_name.clone(),
            user_values: i.user_values.as_ref().map(|v| v.to_string()),
            status: i.status.to_string(),
            created_at: Some(to_timestamp(i.created_at)),
            plan_id: i.plan_id.map(|id| id.to_string()),
        }
    }
}

impl From<&ManagedServicePlan> for ManagedServicePlanProto {
    fn from(plan: &ManagedServicePlan) -> Self {
        let entitlements: Vec<PlanEntitlement> =
            serde_json::from_value(plan.entitlements.clone()).unwrap_or_default();
        Self {
            id: plan.id.to_string(),
            service_id: plan.service_id.to_string(),
            slug: plan.slug.clone(),
            name: plan.name.clone(),
            description: plan.description.clone(),
            status: plan.status.clone(),
            highlighted: plan.highlighted,
            values_override: plan.values_override.as_ref().map(|v| v.to_string()),
            entitlements: entitlements
                .iter()
                .map(|e| ManagedServicePlanEntitlementProto {
                    key: e.key.clone(),
                    label: e.label.clone(),
                    value: e.value.clone(),
                })
                .collect(),
            price_monthly_cents: plan.price_monthly_cents,
            price_yearly_cents: plan.price_yearly_cents,
            created_at: Some(to_timestamp(plan.created_at)),
            stripe_price_id_monthly: plan.stripe_price_id_monthly.clone(),
            stripe_price_id_yearly: plan.stripe_price_id_yearly.clone(),
            requires_payment: plan.requires_payment,
        }
    }
}

fn managed_error_to_status(err: ManagedServiceError) -> Status {
    let message = err.to_string();
    match err {
        ManagedServiceError::Authorization(_) => Status::permission_denied(message),
        ManagedServiceError::Database(_)
        | ManagedServiceError::Fabrique(_)
        | ManagedServiceError::Workflow(_) => {
            tracing::error!(error = %message, "internal managed service error");
            Status::internal("internal error")
        }
        ManagedServiceError::ServiceNotFound(_)
        | ManagedServiceError::VersionNotFound(_)
        | ManagedServiceError::InstanceNotFound(_)
        | ManagedServiceError::OrganizationNotFound(_)
        | ManagedServiceError::ProjectNotFound(_) => Status::not_found(message),
        ManagedServiceError::VersionAlreadyExists(_) => Status::already_exists(message),
        ManagedServiceError::NamespaceTooLong { .. } => Status::invalid_argument(message),
        ManagedServiceError::InvalidInstanceStatus(..) => Status::failed_precondition(message),
        ManagedServiceError::InvalidDeployTarget(..) => Status::failed_precondition(message),
        ManagedServiceError::MissingDeployTarget(_)
        | ManagedServiceError::NoClusterMatchingDeployTarget(_) => {
            Status::failed_precondition(message)
        }
        ManagedServiceError::PlanNotFound(_) => Status::not_found(message),
        ManagedServiceError::PlanNotActive(_) | ManagedServiceError::PlanRequiresPayment(_) => {
            Status::failed_precondition(message)
        }
        ManagedServiceError::PlanServiceMismatch { .. } => Status::invalid_argument(message),
    }
}

fn parse_json(value: &Option<String>) -> Result<Option<Value>, Error> {
    value
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| Error::InvalidInput(format!("invalid JSON: {e}")))
}

#[tonic::async_trait]
impl<A: Authorize + 'static> managed_services_server::ManagedServices for ManagedServicesRpc<A> {
    async fn list_services(
        &self,
        _request: Request<ListServicesRequest>,
    ) -> Result<Response<ListServicesResponse>, Status> {
        let services = self
            .service
            .list_services()
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(ListServicesResponse {
            services: services.iter().map(Into::into).collect(),
        }))
    }

    async fn get_service(
        &self,
        request: Request<GetServiceRequest>,
    ) -> Result<Response<GetServiceResponse>, Status> {
        let slug = request.into_inner().slug;

        if slug.is_empty() {
            return Err(Error::InvalidInput("slug is required".to_owned()))?;
        }

        let service = self
            .service
            .find_service_by_slug(&slug)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(GetServiceResponse {
            service: Some((&service).into()),
        }))
    }

    async fn list_versions(
        &self,
        request: Request<ListVersionsRequest>,
    ) -> Result<Response<ListVersionsResponse>, Status> {
        let service_slug = request.into_inner().service_slug;

        if service_slug.is_empty() {
            return Err(Error::InvalidInput("service_slug is required".to_owned()))?;
        }

        let versions = self
            .service
            .list_versions(&service_slug)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(ListVersionsResponse {
            versions: versions.iter().map(Into::into).collect(),
        }))
    }

    async fn list_plans(
        &self,
        request: Request<ListPlansRequest>,
    ) -> Result<Response<ListPlansResponse>, Status> {
        let service_slug = request.into_inner().service_slug;

        if service_slug.is_empty() {
            return Err(Error::InvalidInput("service_slug is required".to_owned()))?;
        }

        let service = self
            .service
            .find_service_by_slug(&service_slug)
            .await
            .map_err(managed_error_to_status)?;

        let plans = self
            .service
            .list_plans(service.id)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(ListPlansResponse {
            plans: plans.iter().map(Into::into).collect(),
        }))
    }

    async fn register_version(
        &self,
        request: Request<RegisterVersionRequest>,
    ) -> Result<Response<RegisterVersionResponse>, Status> {
        self.authenticate_ci(&request)?;

        let req = request.into_inner();

        if req.service_slug.is_empty() {
            return Err(Error::InvalidInput("service_slug is required".to_owned()))?;
        }
        if req.oci_reference.is_empty() || !req.oci_reference.starts_with("oci://") {
            return Err(Error::InvalidInput(
                "oci_reference must start with oci://".to_owned(),
            ))?;
        }

        let schema_value = req
            .configurable_values_schema
            .as_deref()
            .map(serde_json::from_str::<Value>)
            .transpose()
            .map_err(|e| {
                Error::InvalidInput(format!("invalid configurable_values_schema JSON: {e}"))
            })?;

        let ui_schema_value = req
            .ui_schema
            .as_deref()
            .map(serde_json::from_str::<Value>)
            .transpose()
            .map_err(|e| Error::InvalidInput(format!("invalid ui_schema JSON: {e}")))?;

        let connection_info_value = req
            .connection_info_schema
            .as_deref()
            .map(serde_json::from_str::<Value>)
            .transpose()
            .map_err(|e| {
                Error::InvalidInput(format!("invalid connection_info_schema JSON: {e}"))
            })?;

        let mut tx = self.service.begin().await.map_err(Error::from)?;

        let version = self
            .service
            .register_version(
                &mut tx,
                &req.service_slug,
                &req.chart_version,
                req.app_version.as_deref(),
                &req.oci_reference,
                schema_value.as_ref(),
                ui_schema_value.as_ref(),
                connection_info_value.as_ref(),
            )
            .await
            .map_err(managed_error_to_status)?;

        tx.commit().await.map_err(Error::from)?;

        Ok(Response::new(RegisterVersionResponse {
            version: Some((&version).into()),
        }))
    }

    async fn sync_plans(
        &self,
        request: Request<SyncPlansRequest>,
    ) -> Result<Response<SyncPlansResponse>, Status> {
        self.authenticate_ci(&request)?;

        let req = request.into_inner();

        if req.service_slug.is_empty() {
            return Err(Error::InvalidInput("service_slug is required".to_owned()))?;
        }

        let service = self
            .service
            .find_service_by_slug(&req.service_slug)
            .await
            .map_err(managed_error_to_status)?;

        let mut tx = self.service.begin().await.map_err(Error::from)?;

        let mut synced = Vec::new();
        for entry in &req.plans {
            if entry.slug.is_empty() {
                return Err(Error::InvalidInput("plan slug is required".to_owned()))?;
            }

            let values_override = parse_json(&entry.values_override)?;
            let entitlements: Value = serde_json::to_value(
                entry
                    .entitlements
                    .iter()
                    .map(|e| PlanEntitlement {
                        key: e.key.clone(),
                        label: e.label.clone(),
                        value: e.value.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| Error::InvalidInput(format!("invalid entitlements: {e}")))?;

            let status = if entry.status.is_empty() {
                "active"
            } else {
                &entry.status
            };

            let plan = self
                .service
                .upsert_plan(
                    &mut tx,
                    service.id,
                    &entry.slug,
                    &entry.name,
                    entry.description.as_deref(),
                    status,
                    entry.highlighted,
                    values_override.as_ref(),
                    &entitlements,
                    entry.price_monthly_cents,
                    entry.price_yearly_cents,
                    entry.stripe_price_id_monthly.as_deref(),
                    entry.stripe_price_id_yearly.as_deref(),
                    entry.requires_payment.unwrap_or(true),
                )
                .await
                .map_err(managed_error_to_status)?;

            synced.push(plan);
        }

        tx.commit().await.map_err(Error::from)?;

        Ok(Response::new(SyncPlansResponse {
            plans: synced.iter().map(Into::into).collect(),
        }))
    }

    async fn create_instance(
        &self,
        request: Request<CreateInstanceRequest>,
    ) -> Result<Response<CreateInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        if req.project_slug.is_empty() {
            return Err(Error::InvalidInput("project_slug is required".to_owned()))?;
        }
        let version_id = req
            .version_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId(req.version_id))?;
        if req.plan_id.is_empty() {
            return Err(Error::InvalidInput("plan_id is required".to_owned()))?;
        }
        let plan_id = req
            .plan_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId(req.plan_id))?;
        let user_values = parse_json(&req.user_values)?;
        let secret_values = parse_json(&req.secret_values)?;

        let scheduler = ManagedWorkflowScheduler;
        let mut tx = self.pool.begin().await.map_err(Error::from)?;

        let instance = self
            .service
            .clone()
            .create_instance(
                &principal,
                &mut tx,
                &scheduler,
                frn_core::managed::CreateInstanceRequest {
                    project_slug: req.project_slug,
                    organization_slug: req.organization_slug,
                    service_slug: req.service_slug,
                    version_id,
                    plan_id,
                    user_values,
                    secret_values,
                },
            )
            .await
            .map_err(managed_error_to_status)?;

        tx.commit().await.map_err(Error::from)?;

        let view = self
            .service
            .find_instance_with_status(instance.id)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(CreateInstanceResponse {
            instance: Some((&view).into()),
        }))
    }

    async fn list_instances(
        &self,
        request: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let project_slug = request.into_inner().project_slug;

        if project_slug.is_empty() {
            return Err(Error::InvalidInput("project_slug is required".to_owned()))?;
        }

        let instances = self
            .service
            .list_instances_checked(&principal, &project_slug)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(ListInstancesResponse {
            instances: instances.iter().map(Into::into).collect(),
        }))
    }

    async fn get_instance(
        &self,
        request: Request<GetInstanceRequest>,
    ) -> Result<Response<GetInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let instance_id = request
            .into_inner()
            .instance_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId("instance_id".to_owned()))?;

        let instance = self
            .service
            .get_instance_checked(&principal, instance_id)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(GetInstanceResponse {
            instance: Some((&instance).into()),
        }))
    }

    async fn upgrade_instance(
        &self,
        request: Request<UpgradeInstanceRequest>,
    ) -> Result<Response<UpgradeInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        let instance_id = req
            .instance_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId(req.instance_id))?;
        let version_id = req
            .version_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId(req.version_id))?;
        let user_values = parse_json(&req.user_values)?;
        let secret_values = parse_json(&req.secret_values)?;

        let scheduler = ManagedWorkflowScheduler;
        let mut tx = self.pool.begin().await.map_err(Error::from)?;

        self.service
            .clone()
            .upgrade_instance(
                &principal,
                &mut tx,
                &scheduler,
                frn_core::managed::UpgradeInstanceRequest {
                    instance_id,
                    version_id,
                    user_values,
                    secret_values,
                },
            )
            .await
            .map_err(managed_error_to_status)?;

        tx.commit().await.map_err(Error::from)?;

        let view = self
            .service
            .find_instance_with_status(instance_id)
            .await
            .map_err(managed_error_to_status)?;

        Ok(Response::new(UpgradeInstanceResponse {
            instance: Some((&view).into()),
        }))
    }

    async fn delete_instance(
        &self,
        request: Request<DeleteInstanceRequest>,
    ) -> Result<Response<DeleteInstanceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let instance_id = request
            .into_inner()
            .instance_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId("instance_id".to_owned()))?;

        let scheduler = ManagedWorkflowScheduler;
        let mut tx = self.pool.begin().await.map_err(Error::from)?;

        self.service
            .clone()
            .delete_instance(&principal, &mut tx, &scheduler, instance_id)
            .await
            .map_err(managed_error_to_status)?;

        tx.commit().await.map_err(Error::from)?;

        Ok(Response::new(DeleteInstanceResponse {}))
    }

    async fn get_instance_connection_info(
        &self,
        request: Request<GetInstanceConnectionInfoRequest>,
    ) -> Result<Response<GetInstanceConnectionInfoResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let instance_id = request
            .into_inner()
            .instance_id
            .parse::<Uuid>()
            .map_err(|_| Error::MalformedId("instance_id".to_owned()))?;

        let instance = self
            .service
            .get_instance_for_connection_info(&principal, instance_id)
            .await
            .map_err(managed_error_to_status)?;

        let version = self
            .service
            .find_version_by_id(instance.version_id)
            .await
            .map_err(managed_error_to_status)?;

        let schema = version.connection_info_schema.ok_or_else(|| {
            Status::not_found("no connection_info_schema defined for this service version")
        })?;

        let fields = self
            .resolve_connection_info(
                &schema,
                instance.cluster_id,
                &instance.namespace,
                &instance.release_name,
            )
            .await?;

        Ok(Response::new(GetInstanceConnectionInfoResponse { fields }))
    }
}

impl<A: Authorize> ManagedServicesRpc<A> {
    /// Resolves connection info by reading K8s secrets from the hosting cluster
    /// and assembling the fields according to the schema.
    ///
    /// Schema format:
    /// ```json
    /// {
    ///   "secrets": [{ "ref": "{release}-postgres-credentials", "keys": ["username", "password"] }],
    ///   "fields": [
    ///     { "key": "host", "label": "Hôte", "value": "{release}-rw.{namespace}.svc.cluster.local", "display": "text", "order": 1 },
    ///     { "key": "username", "label": "Utilisateur", "source": "{release}-postgres-credentials/username", "display": "text", "order": 3 },
    ///     { "key": "connectionString", "label": "Connection string", "template": "postgresql://{username}:{password}@{host}:{port}/{database}", "display": "copy", "order": 6 }
    ///   ]
    /// }
    /// ```
    async fn resolve_connection_info(
        &self,
        schema: &Value,
        cluster_id: Uuid,
        namespace: &str,
        release_name: &str,
    ) -> Result<Vec<ConnectionInfoField>, Status> {
        use frn_core::kubernetes::KubernetesClusters;
        use k8s_openapi::api::core::v1::Secret;
        use kube::api::Api;

        let clusters = KubernetesClusters::new(self.pool.clone(), self.kek.clone());
        let kubeconfig = clusters.decrypt_kubeconfig(cluster_id).await.map_err(|e| {
            tracing::error!(cluster_id = %cluster_id, error = %e, "failed to decrypt kubeconfig");
            Status::internal("internal error")
        })?;
        let kube_client = KubernetesClusters::client_from_kubeconfig(&kubeconfig)
            .await
            .map_err(|e| {
                tracing::error!(cluster_id = %cluster_id, error = %e, "failed to build kube client");
                Status::internal("internal error")
            })?;

        let secrets_api: Api<Secret> = Api::namespaced(kube_client, namespace);

        let secrets_spec = schema
            .get("secrets")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        let mut secret_data: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for spec in &secrets_spec {
            let secret_ref = spec
                .get("ref")
                .and_then(|r| r.as_str())
                .unwrap_or_default()
                .replace("{release}", release_name);

            let k8s_secret = secrets_api.get(&secret_ref).await.map_err(|e| {
                tracing::error!(secret_ref = %secret_ref, error = %e, "failed to read k8s secret");
                Status::internal("internal error")
            })?;

            let mut entries = BTreeMap::new();
            if let Some(data) = k8s_secret.data {
                for (key, value) in &data {
                    let decoded = String::from_utf8(value.0.clone()).unwrap_or_default();
                    entries.insert(key.clone(), decoded);
                }
            }
            secret_data.insert(secret_ref, entries);
        }

        let fields_spec = schema
            .get("fields")
            .and_then(|f| f.as_array())
            .ok_or_else(|| Status::internal("connection_info_schema missing 'fields' array"))?;

        let mut resolved: BTreeMap<String, String> = BTreeMap::new();
        let mut fields: Vec<ConnectionInfoField> = Vec::new();

        for field in fields_spec {
            let key = field.get("key").and_then(|k| k.as_str()).unwrap_or("");
            let label = field.get("label").and_then(|l| l.as_str()).unwrap_or(key);
            let display = field
                .get("display")
                .and_then(|d| d.as_str())
                .unwrap_or("text");
            let order = field.get("order").and_then(|o| o.as_i64()).unwrap_or(0) as i32;

            let value = if let Some(source) = field.get("source").and_then(|s| s.as_str()) {
                let source = source.replace("{release}", release_name);
                let parts: Vec<&str> = source.splitn(2, '/').collect();
                if parts.len() == 2 {
                    secret_data
                        .get(parts[0])
                        .and_then(|s| s.get(parts[1]))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            } else if let Some(static_val) = field.get("value").and_then(|v| v.as_str()) {
                interpolate(static_val, release_name, namespace, &resolved)
            } else {
                String::new()
            };

            resolved.insert(key.to_owned(), value.clone());

            fields.push(ConnectionInfoField {
                key: key.to_owned(),
                label: label.to_owned(),
                value,
                display: display.to_owned(),
                order,
            });
        }

        for field in &mut fields {
            if let Some(template) = fields_spec
                .iter()
                .find(|f| f.get("key").and_then(|k| k.as_str()) == Some(&field.key))
                .and_then(|f| f.get("template"))
                .and_then(|t| t.as_str())
            {
                field.value = interpolate(template, release_name, namespace, &resolved);
            }
        }

        fields.sort_by_key(|f| f.order);

        Ok(fields)
    }
}

fn interpolate(
    template: &str,
    release_name: &str,
    namespace: &str,
    resolved: &BTreeMap<String, String>,
) -> String {
    let mut result = template
        .replace("{release}", release_name)
        .replace("{namespace}", namespace);

    for (key, value) in resolved {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}
