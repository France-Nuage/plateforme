#![allow(dead_code)]

use auth::mock::WithWellKnown;
use fabrique::{Factory, Persist, SoftDelete};
use frn_core::identity::ServiceAccount;
use frn_core::managed::{
    DeployManagedServiceParams, ManagedService, ManagedServiceCategory, ManagedServiceInstance,
    ManagedServiceVersion, ManagedServices,
};
use frn_core::workflow::WorkflowScheduler;
use spicedb::SpiceDB;
use sqlx::PgConnection;

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
use frn_rpc::v1::compute::instances_client::InstancesClient;
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
    pub resourcemanager: ResourceManager,
    pub workflow: Workflow,
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
            resourcemanager: ResourceManager::create(&server_url).await?,
            workflow: Workflow::create(&server_url).await?,
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

/// Seeds a managed service in the database for testing.
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
    version_id: Uuid,
    service_slug: &str,
) -> ManagedServiceInstance {
    let auth = SpiceDB::mock().await;
    let principal = ServiceAccount::default();
    let scheduler = NoopWorkflowScheduler;
    let mut managed = ManagedServices::new(
        auth,
        pool.clone(),
        frn_core::managed::PlatformConfig {
            default_storage_class: None,
        },
    );
    let mut conn = pool.acquire().await.expect("could not acquire connection");
    managed
        .create_instance(
            &principal,
            &mut conn,
            &scheduler,
            frn_core::managed::CreateInstanceRequest {
                project_id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                service_slug: service_slug.to_owned(),
                version_id,
                user_values: None,
                secret_values: None,
            },
        )
        .await
        .expect("could not seed managed service instance")
}

/// Deactivates a managed service in the database for testing.
pub async fn deactivate_managed_service(pool: &Pool<Postgres>, service_id: Uuid) {
    ManagedService::soft_destroy(pool, service_id)
        .await
        .expect("could not deactivate managed service");
}
