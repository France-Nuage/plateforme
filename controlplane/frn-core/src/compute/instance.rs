use crate::Error;
use crate::authorization::{AuthorizationServer, Permission, Principal, Resource};
use crate::compute::HypervisorFactory;
use crate::resourcemanager::ProjectFactory;
use chrono::{DateTime, Utc};
use database::{Factory, Persistable, Repository};
use hypervisor::instance::Status;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Repository, Resource)]
pub struct Instance {
    /// Unique identifier for the instance
    #[repository(primary)]
    pub id: Uuid,
    /// The hypervisor this instance is attached to
    #[factory(relation = "HypervisorFactory")]
    pub hypervisor_id: Uuid,
    /// The project this instance belongs to
    #[factory(relation = "ProjectFactory")]
    pub project_id: Uuid,
    /// The zero trust network this instance belongs to
    pub zero_trust_network_id: Option<Uuid>,
    /// ID used by the hypervisor to identify this instance remotely
    pub distant_id: String,
    /// Current CPU utilization as a percentage (0.0-100.0)
    pub cpu_usage_percent: f64,
    /// Current disk utilization (in bytes, cannot exceed max_disk_bytes)
    pub disk_usage_bytes: i64,
    /// IP address v4
    pub ip_v4: String,
    /// Maximum CPU cores available to the instance (max 99)
    pub max_cpu_cores: i32,
    /// Maximum disk available to the instance (in bytes, max 100TB)
    pub max_disk_bytes: i64,
    /// Maximum memory available to the instance (in bytes, max 64GB)
    pub max_memory_bytes: i64,
    /// Current memory utilization (in bytes, cannot exceed max_memory_bytes)
    pub memory_usage_bytes: i64,
    /// Human-readable name, defined on the instance
    pub name: String,
    /// Current operational status of the instance
    #[sqlx(try_from = "String")]
    pub status: Status,
    // Creation time of the instance
    pub created_at: DateTime<Utc>,
    // Time of the instance last update
    pub updated_at: DateTime<Utc>,
}

pub struct InstanceCreateRequest {
    /// The instance unique id.
    pub id: String,

    /// The number of cores per socket.
    pub cores: u8,

    /// The disk image to create the instance from.
    pub disk_image: String,

    /// Memory properties.
    pub memory: u32,

    /// The instance human-readable name.
    pub name: String,

    /// The Cloud-Init snippet.
    pub snippet: String,
}

/// Service for managing compute instances.
pub struct Instances<Auth: AuthorizationServer> {
    auth: Auth,
    _db: Pool<Postgres>,
}

impl<Auth: AuthorizationServer> Instances<Auth> {
    /// Creates a new instances service.
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { auth, _db: db }
    }

    /// Lists all instances accessible to the principal.
    pub async fn list<P: Principal + Sync>(
        &mut self,
        _principal: &P,
    ) -> Result<Vec<Instance>, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::List)
        //     .over(&Instance::any())
        //     .check()
        //     .await?;
        todo!()
    }

    /// Creates a new instance.
    pub async fn create<P: Principal + Sync>(
        &mut self,
        principal: &P,
        _request: InstanceCreateRequest,
    ) -> Result<Instance, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Create)
            .over(&Instance::any())
            .await?;
        todo!()
    }

    /// Deletes an instance.
    pub async fn delete<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<Instance, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over(&Instance::some(id))
            .await?;
        todo!()
    }

    /// Starts a stopped instance.
    pub async fn start<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<Instance, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Start)
            .over(&Instance::some(id))
            .await?;
        todo!()
    }

    /// Stops a running instance.
    pub async fn stop<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<Instance, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Stop)
            .over(&Instance::some(id))
            .await?;
        todo!()
    }
}
