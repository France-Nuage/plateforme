#![allow(dead_code)]

use auth::mock::WithWellKnown;
use fabrique::{Factory, Persist, Query, SoftDelete};
use frn_core::identity::ServiceAccount;
use frn_core::managed::{
    DeployManagedServiceParams, ManagedService, ManagedServiceCategory, ManagedServiceInstance,
    ManagedServiceVersion, ManagedServices,
};
use frn_core::workflow::WorkflowScheduler;
use serde_json::json;
use spicedb::SpiceDB;
use sqlx::PgConnection;
use std::sync::Arc;

use async_trait::async_trait;
use auth::OpenID;
use frn_core::identity::User;
use frn_core::kubernetes::{
    ClusterHealthChecker, ClusterHealthError, ClusterHealthInfo, CreateClusterInput,
    KubernetesCluster, KubernetesClusterLabel, KubernetesClusters, KubernetesLabel,
};
use frn_core::resourcemanager::{Organization, Project};
use frn_crypto::Kek;
use frn_rpc::v1::compute::instances_client::InstancesClient;
use frn_rpc::v1::iam::profile_client::ProfileClient;
use frn_rpc::v1::kubernetes::kubernetes_clusters_client::KubernetesClustersClient;
use frn_rpc::v1::managed::managed_services_client::ManagedServicesClient;
use frn_rpc::v1::workflow::workflow_engine_client::WorkflowEngineClient;
use frn_rpc::v1::{
    compute::hypervisors_client::HypervisorsClient,
    resourcemanager::{organizations_client::OrganizationsClient, projects_client::ProjectsClient},
};
use hypervisor::mock::{
    WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock, WithVMCloneMock,
    WithVMCreateMock, WithVMDeleteMock, WithVMDiskResizeMock, WithVMStatusReadMock,
    WithVMStatusStartMock, WithVMStatusStopMock,
};
use mock_server::MockServer;
use server::{Config, error::Error};
use sqlx::types::chrono::Utc;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use tokio::sync::oneshot;
use tonic::{Request, metadata::MetadataValue, transport::Channel};
use uuid::Uuid;

#[derive(Clone)]
struct NoopWorkflowScheduler;

impl WorkflowScheduler<DeployManagedServiceParams> for NoopWorkflowScheduler {
    async fn schedule(
        &self,
        _conn: &mut PgConnection,
        _params: DeployManagedServiceParams,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// gRPC clients for compute services.
#[allow(dead_code)]
pub struct Compute {
    pub hypervisors: HypervisorsClient<Channel>,
    pub instances: InstancesClient<Channel>,
}

impl Compute {
    pub async fn create(dst: &str) -> Result<Self, Error> {
        let hypervisors = HypervisorsClient::connect(dst.to_owned()).await?;
        let instances = InstancesClient::connect(dst.to_owned()).await?;

        Ok(Self {
            hypervisors,
            instances,
        })
    }
}

#[allow(dead_code)]
pub struct ResourceManager {
    pub organizations: OrganizationsClient<Channel>,
    pub projects: ProjectsClient<Channel>,
}

impl ResourceManager {
    pub async fn create(dst: &str) -> Result<Self, Error> {
        let organizations = OrganizationsClient::connect(dst.to_owned()).await?;
        let projects = ProjectsClient::connect(dst.to_owned()).await?;

        Ok(Self {
            organizations,
            projects,
        })
    }
}

pub struct Managed {
    pub services: ManagedServicesClient<Channel>,
}

impl Managed {
    pub async fn create(dst: &str) -> Result<Self, Error> {
        let services = ManagedServicesClient::connect(dst.to_owned()).await?;
        Ok(Self { services })
    }
}

pub struct Kubernetes {
    pub clusters: KubernetesClustersClient<Channel>,
}

impl Kubernetes {
    pub async fn create(dst: &str) -> Result<Self, Error> {
        let clusters = KubernetesClustersClient::connect(dst.to_owned()).await?;
        Ok(Self { clusters })
    }
}

pub struct Workflow {
    pub engine: WorkflowEngineClient<Channel>,
}

impl Workflow {
    pub async fn create(dst: &str) -> Result<Self, Error> {
        let engine = WorkflowEngineClient::connect(dst.to_owned()).await?;
        Ok(Self { engine })
    }
}

pub const TEST_WORKER_TOKEN: &str = "test-worker-token";
pub const TEST_CI_TOKEN: &str = "test-ci-token";

/// Test API wrapper that manages a gRPC server lifecycle.
#[allow(dead_code)]
pub struct Api {
    pub compute: Compute,
    pub managed: Managed,
    pub kubernetes: Kubernetes,
    pub resourcemanager: ResourceManager,
    pub workflow: Workflow,
    pub profile: ProfileClient<Channel>,
    pub mock_server: MockServer,
    pub service_account: ServiceAccount,
    shutdown: Option<oneshot::Sender<()>>,
}

impl Api {
    /// Starts a test server with an in-memory database and mock authentication.
    pub async fn start(pool: &Pool<Postgres>) -> Result<Self, Error> {
        let mock_server = MockServer::new()
            .await
            .with_cluster_next_id()
            .with_cluster_resource_list()
            .with_task_status_read()
            .with_vm_clone()
            .with_vm_create()
            .with_vm_delete()
            .with_vm_disk_resize()
            .with_vm_status_read()
            .with_vm_status_start()
            .with_vm_status_stop()
            .with_well_known();
        let config = Config::test(pool, &mock_server).await?;
        let server_url = format!("http://{}", config.addr);
        let shutdown = server::serve(config).await?;

        let service_account = ServiceAccount::factory()
            .key("nvki8xsDG6lKng3jXrSX9p7Il3XKs9UBegqzdisT".to_owned())
            .create(pool)
            .await
            .expect("could not create service account");

        Ok(Self {
            compute: Compute::create(&server_url).await?,
            managed: Managed::create(&server_url).await?,
            kubernetes: Kubernetes::create(&server_url).await?,
            resourcemanager: ResourceManager::create(&server_url).await?,
            workflow: Workflow::create(&server_url).await?,
            profile: ProfileClient::connect(server_url.clone()).await?,
            mock_server,
            service_account,
            shutdown: Some(shutdown),
        })
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        // Ensure server shutdown to prevent port conflicts in subsequent tests
        if let Some(tx) = self.shutdown.take() {
            tx.send(()).expect("failed to send shutdown signal");
        }
    }
}

/// Adds authentication headers to gRPC requests.
pub trait OnBehalfOf {
    /// Attaches a service account's bearer token to the request.
    fn on_behalf_of(self, principal: &ServiceAccount) -> Self;
}

impl<T> OnBehalfOf for Request<T> {
    fn on_behalf_of(mut self, principal: &ServiceAccount) -> Self {
        let metadata_value = MetadataValue::from_str(&format!("Bearer {}", &principal.key))
            .expect("could not create metadata value from service account key");
        self.metadata_mut().insert("authorization", metadata_value);

        self
    }
}

/// Adds worker authentication headers to gRPC requests.
pub trait IntoWorker {
    fn into_worker(self) -> Self;
}

impl<T> IntoWorker for Request<T> {
    fn into_worker(mut self) -> Self {
        let metadata_value = MetadataValue::from_str(&format!("Bearer {TEST_WORKER_TOKEN}"))
            .expect("could not create metadata value for worker token");
        self.metadata_mut().insert("authorization", metadata_value);
        self
    }
}

/// Adds CI service token authentication headers to gRPC requests.
pub trait IntoCi {
    fn into_ci(self) -> Self;
}

impl<T> IntoCi for Request<T> {
    fn into_ci(mut self) -> Self {
        let metadata_value = MetadataValue::from_str(&format!("Bearer {TEST_CI_TOKEN}"))
            .expect("could not create metadata value for CI token");
        self.metadata_mut().insert("authorization", metadata_value);
        self
    }
}

/// Attaches a user's OIDC bearer token to a gRPC request.
pub trait WithUser {
    fn with_user(self, token: &str) -> Self;
}

impl<T> WithUser for Request<T> {
    fn with_user(mut self, token: &str) -> Self {
        let metadata_value = MetadataValue::from_str(&format!("Bearer {token}"))
            .expect("could not create metadata value from user token");
        self.metadata_mut().insert("authorization", metadata_value);
        self
    }
}

/// Seeds a platform-admin user and returns a valid OIDC bearer token for it.
///
/// The token is signed with the deterministic mock RSA key, so the test
/// server's IAM resolves it to the seeded admin user.
pub async fn seed_admin_token(pool: &Pool<Postgres>, email: &str) -> String {
    User::factory()
        .id(Uuid::new_v4())
        .email(email.to_owned())
        .is_admin(true)
        .create(pool)
        .await
        .expect("could not seed admin user");
    OpenID::token(email)
}

/// Returns a valid OIDC bearer token for a non-admin user. The user row is
/// created on first authentication with `is_admin = false`.
pub fn non_admin_token(email: &str) -> String {
    OpenID::token(email)
}

/// Reachability checker for seeding: always healthy, never touches the network.
#[derive(Clone)]
struct SeedHealthChecker;

#[async_trait]
impl ClusterHealthChecker for SeedHealthChecker {
    async fn check(&self, _kubeconfig_yaml: &str) -> Result<ClusterHealthInfo, ClusterHealthError> {
        Ok(ClusterHealthInfo {
            api_server_url: "https://cluster.test:6443/".to_owned(),
            kubernetes_version: "v1.32.2".to_owned(),
            platform: "linux/amd64".to_owned(),
        })
    }
}

/// Seeds a healthy Kubernetes cluster in the database for testing.
///
/// Goes through the cluster service (the repository-equivalent) with a stub
/// reachability checker, so no real cluster is contacted and the kubeconfig is
/// stored encrypted exactly as in production.
pub async fn seed_kubernetes_cluster(pool: &Pool<Postgres>, name: &str) -> KubernetesCluster {
    let service = KubernetesClusters::with_health_checker(
        pool.clone(),
        Arc::new(Kek::from_bytes([42u8; 32])),
        Arc::new(SeedHealthChecker),
    );
    let admin = User {
        id: Uuid::new_v4(),
        email: format!("seed-admin-{name}@francenuage.fr"),
        sub: None,
        is_admin: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    service
        .create_cluster(
            &admin,
            CreateClusterInput {
                name: name.to_owned(),
                description: None,
                kubeconfig: "apiVersion: v1\nkind: Config\nclusters: []\n".to_owned(),
            },
        )
        .await
        .expect("could not seed kubernetes cluster")
}

/// Label every seeded managed service requires through its deploy_target, and
/// that [`attach_test_deploy_label`] attaches to seeded clusters.
pub const TEST_DEPLOY_LABEL: (&str, &str) = ("availability", "test");

/// Attaches a (key, value) label to a cluster, creating the label on first
/// use. Inserts directly: the service-layer path is covered by the label
/// service tests. The find-then-insert sequence is race-free because each
/// `#[sqlx::test]` runs against its own database and the helper is called
/// sequentially within a test.
pub async fn attach_cluster_label(pool: &Pool<Postgres>, cluster_id: Uuid, key: &str, value: &str) {
    let existing = KubernetesLabel::query()
        .select()
        .r#where(KubernetesLabel::KEY, "=", key.to_owned())
        .r#where(KubernetesLabel::VALUE, "=", value.to_owned())
        .first(pool)
        .await
        .expect("could not look up kubernetes label");
    let label = match existing {
        Some(label) => label,
        None => KubernetesLabel::factory()
            .key(key.to_owned())
            .value(value.to_owned())
            .system(false)
            .create(pool)
            .await
            .expect("could not seed kubernetes label"),
    };

    let attached = KubernetesClusterLabel::query()
        .select()
        .r#where(KubernetesClusterLabel::CLUSTER_ID, "=", cluster_id)
        .r#where(KubernetesClusterLabel::LABEL_ID, "=", label.id)
        .first(pool)
        .await
        .expect("could not look up kubernetes label attachment");
    if attached.is_none() {
        KubernetesClusterLabel::query()
            .insert()
            .set(KubernetesClusterLabel::ID, Uuid::new_v4())
            .set(KubernetesClusterLabel::CLUSTER_ID, cluster_id)
            .set(KubernetesClusterLabel::LABEL_ID, label.id)
            .execute(pool)
            .await
            .expect("could not attach kubernetes label");
    }
}

/// Attaches the [`TEST_DEPLOY_LABEL`] to a cluster so it matches the
/// deploy_target of services seeded by [`seed_managed_service`].
pub async fn attach_test_deploy_label(pool: &Pool<Postgres>, cluster_id: Uuid) {
    attach_cluster_label(pool, cluster_id, TEST_DEPLOY_LABEL.0, TEST_DEPLOY_LABEL.1).await;
}

/// Seeds a managed service in the database for testing. The service requires
/// the [`TEST_DEPLOY_LABEL`] through its deploy_target; pair it with
/// [`attach_test_deploy_label`] on the hosting cluster.
pub async fn seed_managed_service(
    pool: &Pool<Postgres>,
    slug: &str,
    name: &str,
    category: &str,
) -> Uuid {
    let category = ManagedServiceCategory::from_str(category).expect("invalid category");
    let service = ManagedService::factory()
        .slug(slug.to_owned())
        .name(name.to_owned())
        .category(category)
        .deploy_target(Some(json!({TEST_DEPLOY_LABEL.0: TEST_DEPLOY_LABEL.1})))
        .deactivated_at(None)
        .create(pool)
        .await
        .expect("could not seed managed service");
    service.id
}

/// Seeds a managed service version in the database for testing.
pub async fn seed_managed_service_version(
    pool: &Pool<Postgres>,
    service_id: Uuid,
    chart_version: &str,
    app_version: Option<&str>,
    oci_reference: &str,
) -> Uuid {
    let version = ManagedServiceVersion {
        id: Uuid::new_v4(),
        service_id,
        chart_version: chart_version.to_owned(),
        app_version: app_version.map(str::to_owned),
        oci_reference: oci_reference.to_owned(),
        configurable_values_schema: None,
        ui_schema: None,
        connection_info_schema: None,
        deactivated_at: None,
        created_at: Utc::now(),
    }
    .create(pool)
    .await
    .expect("could not seed managed service version");
    version.id
}

/// Seeds a managed service instance in the database for testing.
pub async fn seed_managed_service_instance(
    pool: &Pool<Postgres>,
    service_id: Uuid,
    version_id: Uuid,
    service_slug: &str,
) -> ManagedServiceInstance {
    let plan_id = seed_managed_service_plan(
        pool,
        service_id,
        &format!("{service_slug}-standard"),
        "Standard",
    )
    .await;

    let organization = Organization::factory()
        .slug("seed-org".to_owned())
        .parent_slug(None)
        .create(pool)
        .await
        .expect("could not seed organization");
    let cluster = seed_kubernetes_cluster(pool, "seed-cluster").await;
    attach_test_deploy_label(pool, cluster.id).await;
    let project = Project::factory()
        .slug("test-project".to_owned())
        .organization_slug(organization.slug.clone())
        .create(pool)
        .await
        .expect("could not seed project");

    let auth = SpiceDB::mock().await;
    let principal = ServiceAccount::default();
    let scheduler = NoopWorkflowScheduler;
    let mut managed = ManagedServices::new(
        auth,
        pool.clone(),
        frn_core::managed::PlatformConfig {
            default_storage_class: None,
            cnpg_backup_enabled: false,
            deployment_labels: std::collections::BTreeMap::new(),
            deployment_annotations: std::collections::BTreeMap::new(),
        },
    );
    let mut conn = pool.acquire().await.expect("could not acquire connection");
    managed
        .create_instance(
            &principal,
            &mut conn,
            &scheduler,
            frn_core::managed::CreateInstanceRequest {
                project_slug: project.slug.clone(),
                organization_slug: organization.slug.clone(),
                service_slug: service_slug.to_owned(),
                version_id,
                plan_id,
                user_values: None,
                secret_values: None,
            },
        )
        .await
        .expect("could not seed managed service instance")
}

/// Seeds a managed service plan in the database for testing.
pub async fn seed_managed_service_plan(
    pool: &Pool<Postgres>,
    service_id: Uuid,
    slug: &str,
    name: &str,
) -> Uuid {
    let row = sqlx::query_as::<_, (Uuid,)>(
        r#"INSERT INTO managed.service_plan
               (id, service_id, slug, name, status, entitlements,
                price_monthly_cents, price_yearly_cents, requires_payment)
           VALUES (gen_random_uuid(), $1, $2, $3, 'active', $4, 999, 10789, false)
           RETURNING id"#,
    )
    .bind(service_id)
    .bind(slug)
    .bind(name)
    .bind(json!([{"key": "support_level", "label": "Support", "value": "Email"}]))
    .fetch_one(pool)
    .await
    .expect("could not seed managed service plan");
    row.0
}

/// Removes all managed services (and cascading dependents) inserted by
/// migrations so tests start from a clean slate.
pub async fn clear_managed_services(pool: &Pool<Postgres>) {
    sqlx::query("DELETE FROM managed.service_plan")
        .execute(pool)
        .await
        .expect("could not clear managed service plans");
    sqlx::query("DELETE FROM managed.service")
        .execute(pool)
        .await
        .expect("could not clear managed services");
}

/// Deactivates a managed service in the database for testing.
pub async fn deactivate_managed_service(pool: &Pool<Postgres>, service_id: Uuid) {
    ManagedService::soft_destroy(pool, service_id)
        .await
        .expect("could not deactivate managed service");
}

/// Builds a [`workflow::WorkerContext`] for exercising workflow operations
/// directly against the test database.
///
/// SpiceDB is the in-process mock and the kube client is constructed lazily
/// against an unroutable address: it never connects, which is fine for the
/// database-backed operations that do not touch Kubernetes.
pub async fn worker_context(pool: &Pool<Postgres>) -> workflow::WorkerContext {
    let kube = kube::Client::try_from(kube::Config::new(
        "https://127.0.0.1:1".parse().expect("valid kube api uri"),
    ))
    .expect("could not build kube client");

    workflow::WorkerContext {
        pool: pool.clone(),
        spicedb: SpiceDB::mock().await,
        kube,
        platform_config: frn_core::managed::PlatformConfig {
            default_storage_class: None,
            cnpg_backup_enabled: false,
            deployment_labels: std::collections::BTreeMap::new(),
            deployment_annotations: std::collections::BTreeMap::new(),
        },
        kek: Arc::new(Kek::from_bytes([42u8; 32])),
        kubeconfig_path: None,
    }
}
