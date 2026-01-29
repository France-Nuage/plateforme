//! Compute instance management and lifecycle operations.
//!
//! Provides the Instance data model and Instances service for creating, managing,
//! and controlling compute instances with authorization checks.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::compute::{Hypervisor, HypervisorFactory, HypervisorIdColumn};
use crate::resourcemanager::{Project, ProjectFactory, ProjectIdColumn};
use base64::Engine;
use chrono::{DateTime, Utc};
use fabrique::{Factory, Model, Persist, Query};
use hypervisor::instance::Instances as HypervisorInstancesTrait;
use hypervisor::instance::Status;
use sqlx::{Pool, Postgres};
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Factory, Model, Resource)]
pub struct Instance {
    /// Unique identifier for the instance
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The hypervisor this instance is attached to
    #[fabrique(belongs_to = Hypervisor)]
    pub hypervisor_id: Uuid,
    /// The project this instance belongs to
    #[fabrique(belongs_to = Project)]
    pub project_id: Uuid,
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
    #[fabrique(as = "String")]
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

    /// The disk size in bytes.
    pub disk_size: u64,

    /// The disk image to create the instance from.
    pub disk_image: String,

    /// Memory properties.
    pub memory: u32,

    /// The instance human-readable name.
    pub name: String,

    /// The Cloud-Init snippet.
    pub snippet: String,
}

#[derive(Clone, Debug)]
pub struct InstanceUpdateRequest {
    /// The instance identifier.
    pub id: Uuid,

    /// The optional new name for the instance.
    pub name: Option<String>,

    /// The optional new project to move the instance to.
    pub project_id: Option<Uuid>,
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

        Instance::all(&self.db).await.map_err(Into::into)
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
        let hypervisors = Hypervisor::all(&self.db).await?;
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

        tracing::info!("next id is: {}", &next_id);

        // Setup Hoop SSH bastion access
        let snippet = self
            .setup_hoop_access(&request.name, request.snippet)
            .await?;

        let instance_id = api
            .create(hypervisor::instance::InstanceCreateRequest {
                id: next_id.clone(),
                cores: request.cores,
                disk_bytes: request.disk_size,
                disk_image: request.disk_image,
                memory_bytes: request.memory,
                name: request.name.clone(),
                snippet,
            })
            .await?;

        let maybe_instance = sqlx::query_as!(
            Instance,
            "SELECT * FROM instances WHERE distant_id = $1 AND hypervisor_id = $2",
            next_id,
            hypervisor.id
        )
        .fetch_optional(&self.db)
        .await?;

        let instance = match maybe_instance {
            None => {
                // Save the created instance in database
                Instance {
                    id: instance_id,
                    hypervisor_id: hypervisor.id,
                    project_id: request.project_id,
                    distant_id: next_id,
                    cpu_usage_percent: 0.0,
                    disk_usage_bytes: 0,
                    ip_v4: String::new(),
                    max_cpu_cores: request.cores as i32,
                    max_disk_bytes: request.disk_size as i64,
                    max_memory_bytes: request.memory as i64,
                    memory_usage_bytes: 0,
                    name: request.name,
                    status: Status::default(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
                .create(&self.db)
                .await?
            }
            Some(instance) => {
                sqlx::query!(
                    "UPDATE instances SET project_id = $1 WHERE id = $2",
                    request.project_id,
                    instance.id
                )
                .execute(&self.db)
                .await?;

                instance
            }
        };

        // Write the relationship synchronously to SpiceDB
        self.auth
            .write_relationship(&Relationship::new(
                &Project::some(request.project_id),
                Relation::Parent,
                &instance,
            ))
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

        // Cleanup Hoop SSH bastion access (best effort)
        self.cleanup_hoop_access(&instance.name).await;

        connector.delete(&instance.distant_id).await?;

        sqlx::query!("DELETE FROM instances WHERE id = $1", instance.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Sets up Hoop SSH bastion access for a new instance.
    ///
    /// Generates an SSH keypair, creates a Hoop agent and connection,
    /// and injects the credentials into the cloud-init snippet.
    async fn setup_hoop_access(
        &self,
        instance_name: &str,
        snippet: String,
    ) -> Result<String, Error> {
        let hoop_api_url = match std::env::var("HOOP_API_URL") {
            Ok(url) => url,
            Err(_) => {
                tracing::warn!("HOOP_API_URL not set, skipping Hoop setup");
                return Ok(snippet);
            }
        };

        let hoop_api_key = match std::env::var("HOOP_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                tracing::warn!("HOOP_API_KEY not set, skipping Hoop setup");
                return Ok(snippet);
            }
        };

        // Generate SSH keypair
        let private_key = PrivateKey::random(&mut rand::thread_rng(), Algorithm::Ed25519)
            .map_err(|e| Error::Other(format!("Failed to generate SSH key: {}", e)))?;
        let public_key = private_key
            .public_key()
            .to_openssh()
            .map_err(|e| Error::Other(format!("Failed to format public key: {}", e)))?;
        let private_key_pem = private_key
            .to_openssh(LineEnding::LF)
            .map_err(|e| Error::Other(format!("Failed to format private key: {}", e)))?;
        let private_key_base64 =
            base64::engine::general_purpose::STANDARD.encode(private_key_pem.as_bytes());

        let client = reqwest::Client::new();

        // Create Hoop agent
        let agent_token =
            hoop::api::create_agent(&hoop_api_url, &client, &hoop_api_key, instance_name)
                .await
                .map_err(|e| Error::Other(format!("Failed to create Hoop agent: {}", e)))?;

        // Get agent UUID (required for creating connection)
        let agent = hoop::api::get_agent(&hoop_api_url, &client, &hoop_api_key, instance_name)
            .await
            .map_err(|e| Error::Other(format!("Failed to get Hoop agent: {}", e)))?;

        // Create Hoop connection with SSH credentials
        hoop::api::create_connection(
            &hoop_api_url,
            &client,
            &hoop_api_key,
            instance_name,
            &agent.id,
            "francenuage",
            &private_key_base64,
        )
        .await
        .map_err(|e| Error::Other(format!("Failed to create Hoop connection: {}", e)))?;

        // Inject credentials into snippet
        let snippet = snippet
            .replace("${HOOP_AGENT_TOKEN}", &agent_token)
            .replace("${HOOP_SSH_PUBLIC_KEY}", &public_key);

        tracing::info!(
            "Hoop SSH bastion access configured for instance {}",
            instance_name
        );

        Ok(snippet)
    }

    /// Cleans up Hoop SSH bastion access for an instance.
    ///
    /// Best effort - errors are logged but don't fail the deletion.
    async fn cleanup_hoop_access(&self, instance_name: &str) {
        let hoop_api_url = match std::env::var("HOOP_API_URL") {
            Ok(url) => url,
            Err(_) => return,
        };

        let hoop_api_key = match std::env::var("HOOP_API_KEY") {
            Ok(key) => key,
            Err(_) => return,
        };

        let client = reqwest::Client::new();

        // Delete connection first
        if let Err(e) =
            hoop::api::delete_connection(&hoop_api_url, &client, &hoop_api_key, instance_name).await
        {
            tracing::warn!(
                "Failed to delete Hoop connection for {}: {}",
                instance_name,
                e
            );
        }

        // Delete agent
        if let Err(e) =
            hoop::api::delete_agent(&hoop_api_url, &client, &hoop_api_key, instance_name).await
        {
            tracing::warn!("Failed to delete Hoop agent for {}: {}", instance_name, e);
        }

        tracing::info!(
            "Hoop SSH bastion access cleaned up for instance {}",
            instance_name
        );
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

        connector.start(&instance.distant_id).await?;

        sqlx::query!(
            "UPDATE instances SET status = $1 WHERE id = $2",
            Status::Running.to_string(),
            instance.id
        )
        .execute(&self.db)
        .await?;

        Ok(())
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

        connector.stop(&instance.distant_id).await?;

        sqlx::query!(
            "UPDATE instances SET status = $1 WHERE id = $2",
            Status::Stopped.to_string(),
            instance.id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Clones an existing instance.
    pub async fn clone_instance<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
        name: Option<String>,
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
            name: name.unwrap_or(existing.name),
            ..existing
        };

        let instance = instance.create(&self.db).await?;

        self.auth
            .write_relationship(&Relationship::new(
                &Project::some(instance.project_id),
                Relation::Parent,
                &instance,
            ))
            .await?;

        Ok(instance)
    }

    /// Updates an existing instance's properties.
    pub async fn update<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: InstanceUpdateRequest,
    ) -> Result<Instance, Error> {
        // Check permission to update the instance
        self.auth
            .can(principal)
            .perform(Permission::Update)
            .over::<Instance>(&request.id)
            .await?;

        // If moving to a new project, check permission to create instances in target project
        if let Some(ref new_project_id) = request.project_id {
            self.auth
                .can(principal)
                .perform(Permission::CreateInstance)
                .over::<Project>(new_project_id)
                .await?;
        }

        let instance = Instance::find_one_by_id(&self.db, request.id).await?;
        let old_project_id = instance.project_id;

        // Build the update query dynamically based on provided fields
        let updated_instance = sqlx::query_as!(
            Instance,
            r#"
            UPDATE instances
            SET
                name = COALESCE($2, name),
                project_id = COALESCE($3, project_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            request.id,
            request.name,
            request.project_id,
        )
        .fetch_one(&self.db)
        .await?;

        // If the project changed, update the authorization relationships
        if let Some(new_project_id) = request.project_id
            && new_project_id != old_project_id
        {
            // Remove old project relationship from SpiceDB
            let old_relationship = Relationship::new(
                &Project::some(old_project_id),
                Relation::Parent,
                &updated_instance,
            );
            self.auth.delete_relationship(&old_relationship).await?;

            // Add new project relationship synchronously to SpiceDB
            self.auth
                .write_relationship(&Relationship::new(
                    &Project::some(new_project_id),
                    Relation::Parent,
                    &updated_instance,
                ))
                .await?;
        }

        Ok(updated_instance)
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
