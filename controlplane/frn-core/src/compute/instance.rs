//! Compute instance management and lifecycle operations.
//!
//! Provides the Instance data model and Instances service for creating, managing,
//! and controlling compute instances with authorization checks.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::compute::{Hypervisor, HypervisorFactory};
use crate::network::{
    AllocateIPRequest, IPAM, InstanceInterface, InterfaceState, SecurityGroup, VNet,
};
use crate::resourcemanager::{Project, ProjectFactory};
use base64::Engine;
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use hypervisor::instance::Instances as HypervisorInstancesTrait;
use hypervisor::instance::Status;
use sqlx::{Pool, Postgres};
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Factory, Persistable, Resource)]
pub struct Instance {
    /// Unique identifier for the instance
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The hypervisor this instance is attached to
    #[fabrique(relation = "Hypervisor", referenced_key = "id")]
    pub hypervisor_id: Uuid,
    /// The project this instance belongs to
    #[fabrique(relation = "Project", referenced_key = "id")]
    pub project_id: Uuid,
    /// The zero trust network this instance belongs to (deprecated, use vpc_id)
    pub zero_trust_network_id: Option<Uuid>,
    /// The VPC this instance belongs to
    pub vpc_id: Option<Uuid>,
    /// The VNet this instance belongs to
    pub vnet_id: Option<Uuid>,
    /// MAC address of the primary network interface
    pub mac_address: Option<String>,
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
        sqlx::query_as::<_, Instance>("SELECT * FROM instances WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }
}

#[derive(Clone, Debug)]
pub struct InstanceCreateRequest {
    /// The project to attach the instance to.
    pub project_id: Uuid,

    /// The VPC this instance belongs to.
    pub vpc_id: Uuid,

    /// The VNet this instance belongs to.
    pub vnet_id: Uuid,

    /// Optional specific IP address to assign within the VNet's subnet.
    pub requested_ip: Option<String>,

    /// Security group IDs to attach to this instance.
    pub security_group_ids: Vec<Uuid>,

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

    /// Optional Cloud-Init snippet (auto-generated if not provided).
    pub snippet: Option<String>,
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

        // Get VNet for network configuration
        let vnet = VNet::find_one_by_id(&self.db, request.vnet_id).await?;

        // Create IPAM service and allocate IP address + MAC
        let mut ipam = IPAM::new(self.auth.clone(), self.db.clone());
        let ip_allocation = ipam
            .allocate(
                principal,
                AllocateIPRequest {
                    vnet_id: request.vnet_id,
                    requested_ip: request.requested_ip.clone(),
                    hostname: Some(request.name.clone()),
                },
            )
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

        // Generate network configuration for cloud-init
        let network_config =
            self.generate_network_config(&ip_allocation.address, Some(&vnet.gateway), &vnet.subnet);

        // Use provided snippet or generate default one with network config
        let base_snippet = request
            .snippet
            .unwrap_or_else(|| self.default_cloud_init_snippet());

        // Setup Hoop SSH bastion access
        let snippet = self.setup_hoop_access(&request.name, base_snippet).await?;

        // Inject network configuration into snippet
        let snippet_with_network = self.inject_network_config(&snippet, &network_config);

        let instance_id = api
            .create(hypervisor::instance::InstanceCreateRequest {
                id: next_id.clone(),
                cores: request.cores,
                disk_bytes: request.disk_size,
                disk_image: request.disk_image,
                memory_bytes: request.memory,
                name: request.name.clone(),
                snippet: snippet_with_network,
            })
            .await?;

        let maybe_instance = sqlx::query_as::<_, Instance>(
            "SELECT * FROM instances WHERE distant_id = $1 AND hypervisor_id = $2",
        )
        .bind(&next_id)
        .bind(hypervisor.id)
        .fetch_optional(&self.db)
        .await?;

        // Extract MAC address (IPAM should always provide one)
        let mac_address = ip_allocation
            .mac_address
            .ok_or_else(|| Error::Other("IP allocation missing MAC address".to_string()))?;

        let instance = match maybe_instance {
            None => {
                // Save the created instance in database with VPC/VNet info
                Instance::factory()
                    .id(instance_id)
                    .hypervisor_id(hypervisor.id)
                    .project_id(request.project_id)
                    .vpc_id(Some(request.vpc_id))
                    .vnet_id(Some(request.vnet_id))
                    .mac_address(Some(mac_address.clone()))
                    .ip_v4(ip_allocation.address.clone())
                    .distant_id(next_id)
                    .max_cpu_cores(request.cores as i32)
                    .max_memory_bytes(request.memory as i64)
                    .name(request.name.clone())
                    .create(&self.db)
                    .await?
            }
            Some(mut instance) => {
                sqlx::query(
                    "UPDATE instances SET project_id = $1, vpc_id = $2, vnet_id = $3, mac_address = $4, ip_v4 = $5 WHERE id = $6"
                )
                .bind(request.project_id)
                .bind(request.vpc_id)
                .bind(request.vnet_id)
                .bind(&mac_address)
                .bind(&ip_allocation.address)
                .bind(instance.id)
                .execute(&self.db)
                .await?;

                instance.vpc_id = Some(request.vpc_id);
                instance.vnet_id = Some(request.vnet_id);
                instance.mac_address = Some(mac_address.clone());
                instance.ip_v4 = ip_allocation.address.clone();
                instance
            }
        };

        // Create InstanceInterface record
        let interface = InstanceInterface::factory()
            .instance_id(instance.id)
            .vnet_id(request.vnet_id)
            .ip_address_id(Some(ip_allocation.id))
            .mac_address(mac_address)
            .device_name("net0".to_string())
            .driver("virtio".to_string())
            .firewall_enabled(true)
            .state(InterfaceState::Active)
            .create(&self.db)
            .await?;

        // Attach security groups to interface
        let security_group_ids = if request.security_group_ids.is_empty() {
            // Get default security group for the VPC
            match SecurityGroup::find_default_for_vpc(&self.db, request.vpc_id).await {
                Ok(Some(default_sg)) => vec![default_sg.id],
                Ok(None) | Err(_) => vec![],
            }
        } else {
            request.security_group_ids
        };

        for sg_id in &security_group_ids {
            sqlx::query(
                "INSERT INTO sg_interface_associations (security_group_id, instance_interface_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(sg_id)
            .bind(interface.id)
            .execute(&self.db)
            .await?;
        }

        // Update IP allocation with interface reference
        sqlx::query("UPDATE ip_addresses SET instance_interface_id = $1 WHERE id = $2")
            .bind(interface.id)
            .bind(ip_allocation.id)
            .execute(&self.db)
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

    /// Generates network configuration for cloud-init.
    fn generate_network_config(
        &self,
        ip_address: &str,
        gateway: Option<&str>,
        subnet: &str,
    ) -> String {
        // Extract prefix length from subnet CIDR
        let prefix = subnet.split('/').nth(1).unwrap_or("24");

        let mut config = format!(
            r#"network:
  version: 2
  ethernets:
    eth0:
      addresses:
        - {}/{}
"#,
            ip_address, prefix
        );

        if let Some(gw) = gateway {
            config.push_str(&format!(
                r#"      routes:
        - to: default
          via: {}
"#,
                gw
            ));
        }

        config
    }

    /// Returns a default cloud-init snippet.
    fn default_cloud_init_snippet(&self) -> String {
        r#"#cloud-config
users:
  - name: francenuage
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    lock_passwd: false
"#
        .to_string()
    }

    /// Injects network configuration into cloud-init snippet.
    fn inject_network_config(&self, snippet: &str, network_config: &str) -> String {
        if snippet.contains("network:") {
            // Network config already present, don't override
            snippet.to_string()
        } else {
            // Append network config to snippet
            format!("{}\n{}", snippet, network_config)
        }
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
        let vpc_ids: Vec<Option<Uuid>> = instances.iter().map(|i| i.vpc_id).collect();
        let vnet_ids: Vec<Option<Uuid>> = instances.iter().map(|i| i.vnet_id).collect();
        let mac_addresses: Vec<Option<String>> =
            instances.iter().map(|i| i.mac_address.clone()).collect();
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

        sqlx::query_as::<_, Instance>(
            r#"
            INSERT INTO instances (id, hypervisor_id, project_id, vpc_id, vnet_id, mac_address, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, ip_v4, disk_usage_bytes, max_disk_bytes)
            SELECT id, hypervisor_id, project_id, vpc_id, vnet_id, mac_address, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, ip_v4, disk_usage_bytes, max_disk_bytes
            FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::uuid[], $5::uuid[], $6::text[], $7::text[], $8::float8[], $9::int4[], $10::int8[], $11::int8[], $12::text[], $13::text[], $14::text[], $15::int8[], $16::int8[]) AS t(id, hypervisor_id, project_id, vpc_id, vnet_id, mac_address, distant_id, cpu_usage_percent, max_cpu_cores, max_memory_bytes, memory_usage_bytes, name, status, ip_v4, disk_usage_bytes, max_disk_bytes)
            ON CONFLICT (id) DO UPDATE
            SET
                hypervisor_id = EXCLUDED.hypervisor_id,
                project_id = EXCLUDED.project_id,
                vpc_id = EXCLUDED.vpc_id,
                vnet_id = EXCLUDED.vnet_id,
                mac_address = EXCLUDED.mac_address,
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
            "#
        )
        .bind(&ids)
        .bind(&hypervisor_ids)
        .bind(&project_ids)
        .bind(&vpc_ids)
        .bind(&vnet_ids)
        .bind(&mac_addresses)
        .bind(&distant_ids)
        .bind(&cpu_usage_percents)
        .bind(&max_cpu_cores)
        .bind(&max_memory_bytes)
        .bind(&memory_usage_bytes)
        .bind(&names)
        .bind(&statuses)
        .bind(&ip_v4s)
        .bind(&disk_usage_bytes)
        .bind(&max_disk_bytes)
        .fetch_all(pool)
        .await
    }
}
