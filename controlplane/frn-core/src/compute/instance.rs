//! Compute instance management and lifecycle operations.
//!
//! Provides the Instance data model and Instances service for creating, managing,
//! and controlling compute instances with authorization checks.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::compute::{Hypervisor, HypervisorFactory};
use crate::resourcemanager::{Project, ProjectFactory};
use chrono::{DateTime, Utc};
use database::{Factory, Persistable, Repository};
use hypervisor::instance::Instances as HypervisorInstancesTrait;
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

impl Instance {
    pub async fn find_one_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<Instance, sqlx::Error> {
        sqlx::query_as!(Instance, "SELECT * FROM instances WHERE id = $1", id)
            .fetch_one(pool)
            .await
    }
}

#[derive(Clone, Debug)]
pub struct InstanceCreateRequest {
    /// The project to attach the instance to.
    pub project_id: Uuid,

    /// The number of cores per socket.
    pub cores: u8,

    /// The disk size.
    pub disk_size: u32,

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
#[derive(Clone, Debug)]
pub struct Instances<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Instances<A> {
    /// Creates a new instances service.
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
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

        tracing::info!("before fetching instances...");

        Instance::list(&self.db).await.map_err(Into::into)
    }

    /// Creates a new instance.
    pub async fn create<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: InstanceCreateRequest,
    ) -> Result<Instance, Error> {
        self.auth
            .can(principal)
            .perform(Permission::CreateInstance)
            .over::<Project>(&request.project_id)
            .await?;

        // Select a hypervisor to deploy the instance on.
        let hypervisors = Hypervisor::list(&self.db).await?;
        let hypervisor = hypervisors
            .first()
            .ok_or_else(|| Error::NoHypervisorsAvailable)?;

        // Create the instance.
        let api = hypervisor::resolve(
            hypervisor.url.clone(),
            hypervisor.authorization_token.clone(),
        );
        let next_id = ::hypervisor::proxmox::api::cluster_next_id(
            &hypervisor.url,
            &reqwest::Client::new(),
            &hypervisor.authorization_token,
        )
        .await
        .map_err(|_| Error::Other("could not get next id".to_owned()))?
        .data
        .to_string();

        let instance_id = api
            .create(hypervisor::instance::InstanceCreateRequest {
                id: next_id.clone(),
                cores: request.cores,
                disk_bytes: request.disk_size,
                disk_image: request.disk_image,
                memory_bytes: request.memory,
                name: request.name.clone(),
                snippet: request.snippet,
            })
            .await?;

        // Save the created instance in database
        let instance = Instance::factory()
            .id(instance_id)
            .hypervisor_id(hypervisor.id)
            .project_id(request.project_id)
            .distant_id(next_id)
            .max_cpu_cores(request.cores as i32)
            .max_memory_bytes(request.memory as i64)
            .name(request.name)
            .create(&self.db)
            .await?;

        // Save the relations
        Relationship::new(
            &Project::some(request.project_id),
            Relation::Parent,
            &instance,
        )
        .publish(&self.db)
        .await?;

        Ok(instance)
    }

    /// Deletes an instance.
    pub async fn delete<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over::<Instance>(&id)
            .await?;

        let instance = Instance::find_one_by_id(&self.db, id).await?;
        let hypervisor = Hypervisor::find_one_by_id(&self.db, instance.hypervisor_id).await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);

        connector
            .delete(&instance.distant_id)
            .await
            .map_err(Into::into)
    }

    /// Starts a stopped instance.
    pub async fn start<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        self.auth
            .can(principal)
            .perform(Permission::Start)
            .over::<Instance>(&id)
            .await?;

        let instance = Instance::find_one_by_id(&self.db, id).await?;
        let hypervisor = Hypervisor::find_one_by_id(&self.db, instance.hypervisor_id).await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);

        connector
            .start(&instance.distant_id)
            .await
            .map_err(Into::into)
    }

    /// Stops a running instance.
    pub async fn stop<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        self.auth
            .can(principal)
            .perform(Permission::Stop)
            .over::<Instance>(&id)
            .await?;

        let instance = Instance::find_one_by_id(&self.db, id).await?;
        let hypervisor = Hypervisor::find_one_by_id(&self.db, instance.hypervisor_id).await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);

        connector
            .stop(&instance.distant_id)
            .await
            .map_err(Into::into)
    }

    /// Stops a running instance.
    pub async fn clone_instance<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<Instance, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Clone)
            .over::<Instance>(&id)
            .await?;

        let existing = Instance::find_one_by_id(&self.db, id).await?;
        let hypervisor = Hypervisor::find_one_by_id(&self.db, existing.hypervisor_id).await?;
        let connector = hypervisor::resolve(hypervisor.url, hypervisor.authorization_token);

        let new_id =
            hypervisor::instance::Instances::clone(&connector, &existing.distant_id).await?;

        let instance = Instance {
            id: Uuid::new_v4(),
            distant_id: new_id,
            ..existing
        };

        let instance = instance.create(&self.db).await?;

        Relationship::new(
            &Project::some(instance.project_id),
            Relation::Parent,
            &instance,
        )
        .publish(&self.db)
        .await?;

        Ok(instance)
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
