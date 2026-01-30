#![allow(dead_code)]

use auth::mock::WithWellKnown;
use fabrique::Factory;
use frn_core::identity::ServiceAccount;
use frn_rpc::v1::compute::instances_client::InstancesClient;
use frn_rpc::v1::{
    compute::hypervisors_client::HypervisorsClient,
    longrunning::operations_client::OperationsClient,
    resourcemanager::{organizations_client::OrganizationsClient, projects_client::ProjectsClient},
};
use hypervisor::mock::{
    WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock, WithVMCloneMock,
    WithVMCreateMock, WithVMDeleteMock, WithVMDiskResizeMock, WithVMStatusReadMock,
    WithVMStatusStartMock, WithVMStatusStopMock,
};
use mock_server::MockServer;
use server::{Config, error::Error};
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use tokio::sync::oneshot;
use tonic::{Request, metadata::MetadataValue, transport::Channel};

/// gRPC clients for compute services.
#[allow(dead_code)]
pub struct Compute {
    pub hypervisors: HypervisorsClient<Channel>,
    pub instances: InstancesClient<Channel>,
}

impl Compute {
    pub async fn create(dst: &String) -> Result<Self, Error> {
        let hypervisors = HypervisorsClient::connect(dst.clone()).await?;
        let instances = InstancesClient::connect(dst.clone()).await?;

        Ok(Self {
            hypervisors,
            instances,
        })
    }
}

#[allow(dead_code)]
pub struct Longrunning {
    pub operations: OperationsClient<Channel>,
}

impl Longrunning {
    pub async fn create(dst: &String) -> Result<Self, Error> {
        let operations = OperationsClient::connect(dst.clone()).await?;

        Ok(Self { operations })
    }
}

#[allow(dead_code)]
pub struct ResourceManager {
    pub organizations: OrganizationsClient<Channel>,
    pub projects: ProjectsClient<Channel>,
}

impl ResourceManager {
    pub async fn create(dst: &String) -> Result<Self, Error> {
        let organizations = OrganizationsClient::connect(dst.clone()).await?;
        let projects = ProjectsClient::connect(dst.clone()).await?;

        Ok(Self {
            organizations,
            projects,
        })
    }
}

/// Test API wrapper that manages a gRPC server lifecycle.
#[allow(dead_code)]
pub struct Api {
    pub compute: Compute,
    pub longrunning: Longrunning,
    pub resourcemanager: ResourceManager,
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
            longrunning: Longrunning::create(&server_url).await?,
            resourcemanager: ResourceManager::create(&server_url).await?,
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
