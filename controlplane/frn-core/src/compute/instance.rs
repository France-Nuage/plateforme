//! Compute instance management and lifecycle operations.
//!
//! Provides the Instance data model and Instances service for creating, managing,
//! and controlling compute instances with authorization checks.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Resource};
use crate::compute::HypervisorFactory;
use crate::resourcemanager::ProjectFactory;
use chrono::{DateTime, Utc};
use database::{Factory, Persistable, Repository};
use hypervisor::instance::Status;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Factory, FromRow, Repository, Resource)]
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
pub struct Instances<Auth: Authorize> {
    auth: Auth,
    _db: Pool<Postgres>,
}

impl<Auth: Authorize> Instances<Auth> {
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
        _principal: &P,
        _request: InstanceCreateRequest,
    ) -> Result<Instance, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&Instance::any())
        //     .await?;
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
            .over::<Instance>(&id)
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
            .over::<Instance>(&id)
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
            .over::<Instance>(&id)
            .await?;
        todo!()
    }
}

impl Instance {
    pub async fn upsert(
        pool: &sqlx::PgPool,
        instances: &[Instance],
    ) -> Result<Vec<Instance>, sqlx::Error> {
        // Extract the data into separate vectors
        let ids: Vec<Uuid> = instances.iter().map(|i| i.id).collect();
        let hypervisor_ids: Vec<Uuid> = instances.iter().map(|i| i.hypervisor_id).collect();
        let project_ids: Vec<Uuid> = instances.iter().map(|i| i.project_id).collect();
        let distant_ids: Vec<String> = instances.iter().map(|i| i.distant_id.clone()).collect();
        let cpu_usage_percents: Vec<f64> = instances.iter().map(|i| i.cpu_usage_percent).collect();
        let max_cpu_cores: Vec<i32> = instances.iter().map(|i| i.max_cpu_cores).collect();
        let max_memory_bytes: Vec<i64> = instances.iter().map(|i| i.max_memory_bytes).collect();
        let memory_usage_bytes: Vec<i64> = instances.iter().map(|i| i.memory_usage_bytes).collect();
        let names: Vec<String> = instances.iter().map(|i| i.name.clone()).collect();
        let statuses: Vec<String> = instances.iter().map(|i| i.status.to_string()).collect();
        let ip_v4s: Vec<String> = instances.iter().map(|i| i.ip_v4.clone()).collect();
        let disk_usage_bytes: Vec<i64> = instances.iter().map(|i| i.disk_usage_bytes).collect();
        let max_disk_bytes: Vec<i64> = instances.iter().map(|i| i.max_disk_bytes).collect();

        sqlx::query_as!(
        Instance,
        r#"
        INSERT INTO instances (id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, ip_v4, disk_usage_bytes, max_disk_bytes)
        SELECT id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, ip_v4, disk_usage_bytes, max_disk_bytes
        FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::text[], $5::float8[], $6::int4[], $7::int8[], $8::int8[], $9::text[], $10::text[], $11::text[], $12::int8[], $13::int8[]) AS t(id, hypervisor_id, project_id, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, ip_v4, disk_usage_bytes, max_disk_bytes)
        ON CONFLICT (id) DO UPDATE
        SET
            hypervisor_id = EXCLUDED.hypervisor_id,
            project_id = EXCLUDED.project_id,
            distant_id = EXCLUDED.distant_id,
            cpu_usage_percent = EXCLUDED.cpu_usage_percent,
            max_cpu_cores = EXCLUDED.max_cpu_cores,
            max_memory_bytes = EXCLUDED.max_memory_bytes,
            memory_usage_bytes = EXCLUDED.memory_usage_bytes,
            name = EXCLUDED.name,
            status = EXCLUDED.status,
            ip_v4 = EXCLUDED.ip_v4,
            disk_usage_bytes = EXCLUDED.disk_usage_bytes,
            max_disk_bytes = EXCLUDED.max_disk_bytes,
            updated_at = NOW()
        RETURNING *
    "#,
        &ids,
        &hypervisor_ids,
        &project_ids,
        &distant_ids,
        &cpu_usage_percents,
        &max_cpu_cores,
        &max_memory_bytes,
        &memory_usage_bytes,
        &names,
        &statuses,
        &ip_v4s,
        &disk_usage_bytes,
        &max_disk_bytes,
    )
    .fetch_all(pool)
    .await
    }
}
