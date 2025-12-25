//! VNet (Virtual Network/Subnet) management.
//!
//! Provides the VNet data model and VNets service for creating, managing,
//! and controlling virtual networks within VPCs with authorization checks.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::network::{IPAllocation, VPC, VPCFactory};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use ipnetwork::IpNetwork;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display as StrumDisplay, EnumString};
use uuid::Uuid;

/// State of a VNet in its lifecycle.
#[derive(Clone, Debug, Default, StrumDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum VNetState {
    #[default]
    Pending,
    Active,
    Error,
}

impl From<VNetState> for String {
    fn from(value: VNetState) -> Self {
        value.to_string()
    }
}

impl From<String> for VNetState {
    fn from(value: String) -> Self {
        VNetState::from_str(&value).unwrap_or_default()
    }
}

/// VNet represents a Virtual Network (subnet) within a VPC.
#[derive(Clone, Debug, Default, Factory, Persistable, Resource)]
pub struct VNet {
    /// Unique identifier for the VNet
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// ID of the parent VPC
    #[fabrique(relation = "VPC", referenced_key = "id")]
    pub vpc_id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Proxmox SDN VNet bridge identifier (e.g., vnet-myapp-prod)
    pub vnet_bridge_id: String,
    /// CIDR notation for the subnet (e.g., 10.0.1.0/24)
    pub subnet: String,
    /// Gateway IP address (e.g., 10.0.1.1)
    pub gateway: String,
    /// Whether DHCP is enabled on this VNet
    pub dhcp_enabled: bool,
    /// Comma-separated list of DNS servers (e.g., 1.1.1.1,8.8.8.8)
    pub dns_servers: String,
    /// Current state of the VNet
    #[fabrique(as = "String")]
    pub state: VNetState,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl VNet {
    /// Find a VNet by its ID.
    pub async fn find_one_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<VNet, sqlx::Error> {
        sqlx::query_as::<_, VNet>("SELECT * FROM vnets WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Find all VNets for a VPC.
    pub async fn find_by_vpc_id(
        pool: &Pool<Postgres>,
        vpc_id: Uuid,
    ) -> Result<Vec<VNet>, sqlx::Error> {
        sqlx::query_as::<_, VNet>("SELECT * FROM vnets WHERE vpc_id = $1")
            .bind(vpc_id)
            .fetch_all(pool)
            .await
    }

    /// Find a VNet by its bridge ID.
    pub async fn find_by_bridge_id(
        pool: &Pool<Postgres>,
        bridge_id: &str,
    ) -> Result<Option<VNet>, sqlx::Error> {
        sqlx::query_as::<_, VNet>("SELECT * FROM vnets WHERE vnet_bridge_id = $1")
            .bind(bridge_id)
            .fetch_optional(pool)
            .await
    }

    /// Calculates the first usable IP address in a subnet (the gateway).
    pub fn calculate_gateway(subnet: &str) -> Result<String, Error> {
        let network: IpNetwork = subnet
            .parse()
            .map_err(|e| Error::Other(format!("Invalid subnet CIDR: {}", e)))?;

        match network {
            IpNetwork::V4(net) => {
                let first_ip = net
                    .nth(1)
                    .ok_or_else(|| Error::Other("Subnet too small for gateway".to_string()))?;
                Ok(first_ip.to_string())
            }
            IpNetwork::V6(net) => {
                // For IPv6, we iterate manually
                let mut iter = net.iter();
                iter.next(); // Skip network address
                let first_ip = iter
                    .next()
                    .ok_or_else(|| Error::Other("Subnet too small for gateway".to_string()))?;
                Ok(first_ip.to_string())
            }
        }
    }

    /// Generates a bridge ID from a VPC slug.
    pub fn generate_bridge_id(vpc_slug: &str) -> String {
        format!("vnet-{}", vpc_slug)
    }
}

/// Request to create a new VNet.
#[derive(Clone, Debug)]
pub struct VNetCreateRequest {
    /// ID of the parent VPC
    pub vpc_id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Subnet in CIDR notation (e.g., 10.0.1.0/24)
    pub subnet: String,
    /// Gateway IP (optional, auto-calculated as first usable IP if empty)
    pub gateway: Option<String>,
    /// Comma-separated DNS servers (optional)
    pub dns_servers: Option<String>,
}

/// Request to update an existing VNet.
#[derive(Clone, Debug)]
pub struct VNetUpdateRequest {
    /// ID of the VNet to update
    pub id: Uuid,
    /// New name (optional)
    pub name: Option<String>,
    /// New DNS servers (optional)
    pub dns_servers: Option<String>,
}

/// Service for managing VNets.
#[derive(Clone, Debug)]
pub struct VNets<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> VNets<A> {
    /// Creates a new VNets service.
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Lists all VNets in a VPC.
    pub async fn list<P: Principal + Sync>(
        &mut self,
        principal: &P,
        vpc_id: Uuid,
    ) -> Result<Vec<VNet>, Error> {
        // First check access to the VPC
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<VPC>(&vpc_id)
            .await?;

        VNet::find_by_vpc_id(&self.db, vpc_id)
            .await
            .map_err(Into::into)
    }

    /// Gets a VNet by ID.
    pub async fn get<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<VNet, Error> {
        let vnet = VNet::find_one_by_id(&self.db, id).await?;

        // Check access to the parent VPC
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        Ok(vnet)
    }

    /// Creates a new VNet.
    pub async fn create<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: VNetCreateRequest,
    ) -> Result<VNet, Error> {
        // Check permission to create VNet in the VPC
        self.auth
            .can(principal)
            .perform(Permission::CreateVNet)
            .over::<VPC>(&request.vpc_id)
            .await?;

        // Get the parent VPC for slug
        let vpc = VPC::find_one_by_id(&self.db, request.vpc_id).await?;

        // Validate subnet CIDR
        let _network: IpNetwork = request
            .subnet
            .parse()
            .map_err(|e| Error::Other(format!("Invalid subnet CIDR: {}", e)))?;

        // Calculate gateway if not provided
        let gateway = match request.gateway {
            Some(gw) if !gw.is_empty() => gw,
            _ => VNet::calculate_gateway(&request.subnet)?,
        };

        // Generate bridge ID
        let vnet_bridge_id = VNet::generate_bridge_id(&vpc.slug);

        // Check bridge ID uniqueness
        if VNet::find_by_bridge_id(&self.db, &vnet_bridge_id)
            .await?
            .is_some()
        {
            return Err(Error::Other(format!(
                "VNet bridge ID already exists: {}",
                vnet_bridge_id
            )));
        }

        // Default DNS servers
        let dns_servers = request
            .dns_servers
            .unwrap_or_else(|| "1.1.1.1,8.8.8.8".to_string());

        // Create VNet
        let vnet = VNet::factory()
            .id(Uuid::new_v4())
            .vpc_id(request.vpc_id)
            .name(request.name)
            .vnet_bridge_id(vnet_bridge_id)
            .subnet(request.subnet)
            .gateway(gateway.clone())
            .dhcp_enabled(false)
            .dns_servers(dns_servers)
            .state(VNetState::Active)
            .create(&self.db)
            .await?;

        // Create authorization relationship
        Relationship::new(&vpc, Relation::Parent, &vnet)
            .publish(&self.db)
            .await?;

        // Reserve the gateway IP in IPAM
        self.reserve_gateway_ip(&vnet, &gateway).await?;

        Ok(vnet)
    }

    /// Reserves the gateway IP address in IPAM.
    async fn reserve_gateway_ip(&self, vnet: &VNet, gateway: &str) -> Result<IPAllocation, Error> {
        let alloc_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ip_addresses (id, vnet_id, address, mac_address, allocation_type, hostname, allocated_at)
            VALUES ($1, $2, $3::inet, NULL, 'GATEWAY', 'gateway', NOW())
            "#
        )
        .bind(alloc_id)
        .bind(vnet.id)
        .bind(gateway)
        .execute(&self.db)
        .await?;

        IPAllocation::find_one_by_id(&self.db, alloc_id)
            .await
            .map_err(Into::into)
    }

    /// Updates an existing VNet.
    pub async fn update<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: VNetUpdateRequest,
    ) -> Result<VNet, Error> {
        let vnet = VNet::find_one_by_id(&self.db, request.id).await?;

        // Check access to the parent VPC
        self.auth
            .can(principal)
            .perform(Permission::UpdateVNet)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        // Build update query with parameterized values to prevent SQL injection
        let mut set_clauses = Vec::new();
        let mut param_index = 2; // $1 is reserved for id

        if request.name.is_some() {
            set_clauses.push(format!("name = ${}", param_index));
            param_index += 1;
        }
        if request.dns_servers.is_some() {
            set_clauses.push(format!("dns_servers = ${}", param_index));
            // param_index += 1; // Uncomment if more fields are added
        }

        if set_clauses.is_empty() {
            return Ok(vnet);
        }

        set_clauses.push("updated_at = NOW()".to_string());
        let query = format!("UPDATE vnets SET {} WHERE id = $1", set_clauses.join(", "));

        let mut query_builder = sqlx::query(&query).bind(request.id);
        if let Some(ref name) = request.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(ref dns_servers) = request.dns_servers {
            query_builder = query_builder.bind(dns_servers);
        }

        query_builder.execute(&self.db).await?;

        VNet::find_one_by_id(&self.db, request.id)
            .await
            .map_err(Into::into)
    }

    /// Deletes a VNet.
    pub async fn delete<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        let vnet = VNet::find_one_by_id(&self.db, id).await?;

        // Check access to the parent VPC
        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        // Check for allocated IP addresses (excluding gateway)
        let ip_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM ip_addresses WHERE vnet_id = $1 AND allocation_type != 'GATEWAY'",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        if ip_count.0 > 0 {
            return Err(Error::Other(
                "Cannot delete VNet with allocated IP addresses".to_string(),
            ));
        }

        // Delete IP allocations (including gateway)
        sqlx::query("DELETE FROM ip_addresses WHERE vnet_id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        // Delete VNet
        sqlx::query("DELETE FROM vnets WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
