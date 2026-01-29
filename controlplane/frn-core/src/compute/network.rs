//! Network management for VPC isolation and IPAM.
//!
//! Provides the Network, Address, and NetworkInterface data models along with
//! the Networks service for creating and managing VPC networks with automatic
//! IP allocation and Proxmox SDN integration.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::compute::{Hypervisor, Instance};
use crate::resourcemanager::{Project, ProjectFactory, ProjectIdColumn};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Model, Persist, Query};
use sqlx::{Pool, Postgres};
use std::net::Ipv4Addr;
use uuid::Uuid;

/// Represents a VPC Network resource.
///
/// Networks connect instances to each other and enable isolation between clients
/// via Proxmox SDN VXLAN overlay.
#[derive(Clone, Debug, Default, Factory, Model, Resource)]
pub struct Network {
    /// Unique identifier for the network
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The project this network belongs to
    #[fabrique(belongs_to = Project)]
    pub project_id: Uuid,
    /// Network name (unique within project)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// CIDR range for the network (e.g., "10.0.0.0/24")
    pub ipv4_range: String,
    /// Gateway IP address
    pub gateway_ipv4: Option<String>,
    /// Maximum Transmission Unit (1300-8896, default 1450)
    pub mtu: i32,
    /// Proxmox SDN Zone ID
    pub proxmox_zone_id: Option<String>,
    /// Proxmox SDN VNet ID
    pub proxmox_vnet_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Network {
    /// Finds a network by ID.
    pub async fn find_one_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<Network, sqlx::Error> {
        sqlx::query_as!(Network, "SELECT * FROM networks WHERE id = $1", id)
            .fetch_one(pool)
            .await
    }

    /// Finds a network by name within a project.
    pub async fn find_by_name(
        pool: &Pool<Postgres>,
        project_id: Uuid,
        name: &str,
    ) -> Result<Option<Network>, sqlx::Error> {
        sqlx::query_as!(
            Network,
            "SELECT * FROM networks WHERE project_id = $1 AND name = $2",
            project_id,
            name
        )
        .fetch_optional(pool)
        .await
    }

    /// Checks if any instances are attached to this network.
    pub async fn has_attached_instances(&self, pool: &Pool<Postgres>) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM instance_network_interfaces WHERE network_id = $1",
            self.id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
        Ok(count > 0)
    }
}

/// Represents an IP address reservation within a network.
#[derive(Clone, Debug, Default, Factory, Model)]
pub struct Address {
    /// Unique identifier for the address
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The network this address belongs to
    #[fabrique(belongs_to = Network)]
    pub network_id: Uuid,
    /// The instance using this address (if IN_USE)
    pub instance_id: Option<Uuid>,
    /// The IPv4 address
    pub address: String,
    /// Optional name for the address
    pub name: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Status: RESERVING, RESERVED, IN_USE
    pub status: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Address status values.
pub mod address_status {
    pub const RESERVING: &str = "RESERVING";
    pub const RESERVED: &str = "RESERVED";
    pub const IN_USE: &str = "IN_USE";
}

impl Address {
    /// Finds an address by its IP within a network.
    pub async fn find_by_ip(
        pool: &Pool<Postgres>,
        network_id: Uuid,
        ip: &str,
    ) -> Result<Option<Address>, sqlx::Error> {
        sqlx::query_as!(
            Address,
            "SELECT * FROM addresses WHERE network_id = $1 AND address = $2",
            network_id,
            ip
        )
        .fetch_optional(pool)
        .await
    }

    /// Finds the next available IP in a network.
    pub async fn find_next_available(
        pool: &Pool<Postgres>,
        network_id: Uuid,
    ) -> Result<Option<Address>, sqlx::Error> {
        sqlx::query_as!(
            Address,
            r#"
            SELECT * FROM addresses
            WHERE network_id = $1 AND status = $2
            ORDER BY address ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
            network_id,
            address_status::RESERVED
        )
        .fetch_optional(pool)
        .await
    }
}

/// Represents a network interface attached to an instance.
#[derive(Clone, Debug, Default, Factory, Model)]
pub struct InstanceNetworkInterface {
    /// Unique identifier
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The instance this interface belongs to
    #[fabrique(belongs_to = Instance)]
    pub instance_id: Uuid,
    /// The network this interface connects to
    #[fabrique(belongs_to = Network)]
    pub network_id: Uuid,
    /// The assigned address
    pub address_id: Option<Uuid>,
    /// Interface name (e.g., "eth0")
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Request to create a new network.
#[derive(Clone, Debug)]
pub struct NetworkInsertRequest {
    /// Project to create the network in
    pub project_id: Uuid,
    /// Network name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// CIDR range (e.g., "10.0.0.0/24")
    pub ipv4_range: String,
    /// Optional MTU (default 1450)
    pub mtu: Option<i32>,
}

/// Service for managing networks.
#[derive(Clone, Debug)]
pub struct Networks<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Networks<A> {
    /// Creates a new networks service.
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Inserts a new network.
    ///
    /// This creates the network in the database and provisions the corresponding
    /// Proxmox SDN resources (Zone, VNet, Subnet).
    pub async fn insert<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: NetworkInsertRequest,
    ) -> Result<Network, Error> {
        // Check permission to create network in project
        self.auth
            .can(principal)
            .perform(Permission::CreateNetwork)
            .over::<Project>(&request.project_id)
            .await?;

        // Parse and validate CIDR
        let (network_addr, prefix_len) = parse_cidr(&request.ipv4_range)?;
        let gateway = calculate_gateway(&network_addr, prefix_len)?;

        // Generate unique IDs for Proxmox resources
        let network_id = Uuid::new_v4();
        let zone_id = format!("zone-{}", &network_id.to_string()[..8]);
        let vnet_id = format!("vnet-{}", &network_id.to_string()[..8]);

        // Create network in database
        let network = Network {
            id: network_id,
            project_id: request.project_id,
            name: request.name,
            description: request.description,
            ipv4_range: request.ipv4_range.clone(),
            gateway_ipv4: Some(gateway.to_string()),
            mtu: request.mtu.unwrap_or(1450),
            proxmox_zone_id: Some(zone_id.clone()),
            proxmox_vnet_id: Some(vnet_id.clone()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
        .create(&self.db)
        .await?;

        // Pre-allocate IP addresses in the range
        self.allocate_ip_range(&network, &network_addr, prefix_len, &gateway)
            .await?;

        // Provision Proxmox SDN resources
        if let Err(e) = self.provision_proxmox_sdn(&network, &zone_id, &vnet_id).await {
            // Rollback: delete network and addresses
            tracing::error!("Failed to provision Proxmox SDN: {}", e);
            sqlx::query!("DELETE FROM networks WHERE id = $1", network.id)
                .execute(&self.db)
                .await?;
            return Err(e);
        }

        // Write authorization relationship
        self.auth
            .write_relationship(&Relationship::new(
                &Project::some(request.project_id),
                Relation::Parent,
                &network,
            ))
            .await?;

        Ok(network)
    }

    /// Gets a network by ID.
    pub async fn get<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<Network, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<Network>(&id)
            .await?;

        Network::find_one_by_id(&self.db, id)
            .await
            .map_err(Into::into)
    }

    /// Lists all networks accessible to the principal.
    pub async fn list<P: Principal + Sync>(
        &mut self,
        principal: &P,
        project_id: Option<Uuid>,
    ) -> Result<Vec<Network>, Error> {
        // TODO: filter by authorization
        let _ = principal;

        match project_id {
            Some(pid) => {
                sqlx::query_as!(Network, "SELECT * FROM networks WHERE project_id = $1", pid)
                    .fetch_all(&self.db)
                    .await
                    .map_err(Into::into)
            }
            None => Network::all(&self.db).await.map_err(Into::into),
        }
    }

    /// Deletes a network.
    ///
    /// Fails if any instances are still attached to the network.
    pub async fn delete<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over::<Network>(&id)
            .await?;

        let network = Network::find_one_by_id(&self.db, id).await?;

        // Check for attached instances
        if network.has_attached_instances(&self.db).await? {
            return Err(Error::NetworkHasAttachedInstances);
        }

        // Deprovision Proxmox SDN resources
        if let Err(e) = self.deprovision_proxmox_sdn(&network).await {
            tracing::warn!("Failed to deprovision Proxmox SDN: {}", e);
            // Continue with deletion anyway
        }

        // Delete network (cascades to addresses)
        sqlx::query!("DELETE FROM networks WHERE id = $1", network.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Pre-allocates IP addresses in a network range.
    async fn allocate_ip_range(
        &self,
        network: &Network,
        network_addr: &Ipv4Addr,
        prefix_len: u8,
        gateway: &Ipv4Addr,
    ) -> Result<(), Error> {
        let host_count = 2u32.pow(32 - prefix_len as u32) - 2; // Exclude network and broadcast
        let start_ip = u32::from(*network_addr) + 1;

        let mut addresses = Vec::new();
        for i in 0..host_count.min(254) {
            // Limit to 254 addresses for now
            let ip = Ipv4Addr::from(start_ip + i);
            if ip == *gateway {
                continue; // Skip gateway address
            }
            addresses.push(Address {
                id: Uuid::new_v4(),
                network_id: network.id,
                instance_id: None,
                address: ip.to_string(),
                name: None,
                description: None,
                status: address_status::RESERVED.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Batch insert addresses
        for address in addresses {
            address.create(&self.db).await?;
        }

        Ok(())
    }

    /// Provisions Proxmox SDN resources for a network.
    async fn provision_proxmox_sdn(
        &self,
        network: &Network,
        zone_id: &str,
        vnet_id: &str,
    ) -> Result<(), Error> {
        // Get a hypervisor to use for SDN operations
        let hypervisors = Hypervisor::all(&self.db).await?;
        let hypervisor = hypervisors
            .first()
            .ok_or_else(|| Error::NoHypervisorsAvailable)?;

        let client = reqwest::Client::new();

        // 1. Create Zone (VXLAN)
        let peers = self.get_hypervisor_peers().await?;
        let zone_config = hypervisor::proxmox::api::SdnZoneConfig {
            zone: zone_id.to_string(),
            zone_type: hypervisor::proxmox::api::SdnZoneType::Vxlan,
            peers: Some(peers),
            mtu: Some(network.mtu as u32),
            vxlan_port: None,
        };
        hypervisor::proxmox::api::sdn_zone_create(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
            &zone_config,
        )
        .await
        .map_err(|e| Error::Other(format!("Failed to create SDN zone: {}", e)))?;

        // 2. Create VNet
        let vnet_tag = self.generate_vni_tag().await?;
        let vnet_config = hypervisor::proxmox::api::SdnVnetConfig {
            vnet: vnet_id.to_string(),
            zone: zone_id.to_string(),
            alias: network.description.clone(),
            tag: Some(vnet_tag),
            vlanaware: None,
        };
        hypervisor::proxmox::api::sdn_vnet_create(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
            &vnet_config,
        )
        .await
        .map_err(|e| Error::Other(format!("Failed to create SDN vnet: {}", e)))?;

        // 3. Create Subnet
        let subnet_config = hypervisor::proxmox::api::SdnSubnetConfig::from_cidr(
            &network.ipv4_range,
            network.gateway_ipv4.clone(),
        );
        hypervisor::proxmox::api::sdn_subnet_create(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
            vnet_id,
            &subnet_config,
        )
        .await
        .map_err(|e| Error::Other(format!("Failed to create SDN subnet: {}", e)))?;

        // 4. Apply SDN configuration
        hypervisor::proxmox::api::sdn_apply(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
        )
        .await
        .map_err(|e| Error::Other(format!("Failed to apply SDN config: {}", e)))?;

        Ok(())
    }

    /// Deprovisions Proxmox SDN resources for a network.
    async fn deprovision_proxmox_sdn(&self, network: &Network) -> Result<(), Error> {
        let hypervisors = Hypervisor::all(&self.db).await?;
        let hypervisor = match hypervisors.first() {
            Some(h) => h,
            None => return Ok(()), // No hypervisors, nothing to deprovision
        };

        let client = reqwest::Client::new();
        let zone_id = network.proxmox_zone_id.as_deref().unwrap_or("");
        let vnet_id = network.proxmox_vnet_id.as_deref().unwrap_or("");

        if zone_id.is_empty() || vnet_id.is_empty() {
            return Ok(()); // No SDN resources to delete
        }

        // 1. Delete Subnet
        let subnet_id = hypervisor::proxmox::api::cidr_to_subnet_id(&network.ipv4_range);
        let _ = hypervisor::proxmox::api::sdn_subnet_delete(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
            vnet_id,
            &subnet_id,
        )
        .await;

        // 2. Delete VNet
        let _ = hypervisor::proxmox::api::sdn_vnet_delete(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
            vnet_id,
        )
        .await;

        // 3. Delete Zone
        let _ = hypervisor::proxmox::api::sdn_zone_delete(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
            zone_id,
        )
        .await;

        // 4. Apply SDN configuration
        let _ = hypervisor::proxmox::api::sdn_apply(
            &hypervisor.url,
            &client,
            &hypervisor.authorization_token,
        )
        .await;

        Ok(())
    }

    /// Gets the comma-separated list of hypervisor IPs for VXLAN peers.
    async fn get_hypervisor_peers(&self) -> Result<String, Error> {
        let hypervisors = Hypervisor::all(&self.db).await?;
        let peers: Vec<String> = hypervisors
            .iter()
            .filter_map(|h| {
                // Extract IP from URL (e.g., "https://10.0.0.1:8006" -> "10.0.0.1")
                url::Url::parse(&h.url)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
            })
            .collect();
        Ok(peers.join(","))
    }

    /// Generates a unique VNI tag for a new network.
    async fn generate_vni_tag(&self) -> Result<u32, Error> {
        // Get the max tag currently in use and add 1
        let max_tag: Option<i32> = sqlx::query_scalar!(
            r#"
            SELECT MAX(CAST(NULLIF(
                regexp_replace(proxmox_vnet_id, '[^0-9]', '', 'g'),
                ''
            ) AS INTEGER))
            FROM networks
            WHERE proxmox_vnet_id IS NOT NULL
            "#
        )
        .fetch_one(&self.db)
        .await?;

        // Start from 100 and increment
        Ok(max_tag.map(|t| t as u32 + 1).unwrap_or(100))
    }
}

/// Parses a CIDR string into network address and prefix length.
fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, u8), Error> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidCidr(cidr.to_string()));
    }

    let addr: Ipv4Addr = parts[0]
        .parse()
        .map_err(|_| Error::InvalidCidr(cidr.to_string()))?;
    let prefix: u8 = parts[1]
        .parse()
        .map_err(|_| Error::InvalidCidr(cidr.to_string()))?;

    if prefix > 32 {
        return Err(Error::InvalidCidr(cidr.to_string()));
    }

    Ok((addr, prefix))
}

/// Calculates the gateway address (first usable IP in the range).
fn calculate_gateway(network_addr: &Ipv4Addr, _prefix_len: u8) -> Result<Ipv4Addr, Error> {
    let addr_u32 = u32::from(*network_addr);
    Ok(Ipv4Addr::from(addr_u32 + 1))
}

/// IPAM (IP Address Management) functions.
pub mod ipam {
    use super::*;

    /// Allocates an IP address for an instance in a network.
    ///
    /// If `requested_ip` is provided, attempts to allocate that specific IP.
    /// Otherwise, allocates the next available IP.
    pub async fn allocate_ip(
        pool: &Pool<Postgres>,
        network_id: Uuid,
        instance_id: Uuid,
        requested_ip: Option<&str>,
    ) -> Result<Address, Error> {
        let address = match requested_ip {
            Some(ip) => {
                // Check if the requested IP is available
                let addr = Address::find_by_ip(pool, network_id, ip)
                    .await?
                    .ok_or_else(|| Error::IpNotInRange(ip.to_string()))?;

                if addr.status == address_status::IN_USE {
                    return Err(Error::IpAlreadyInUse(ip.to_string()));
                }

                addr
            }
            None => {
                // Get next available IP
                Address::find_next_available(pool, network_id)
                    .await?
                    .ok_or_else(|| Error::NoAvailableIps)?
            }
        };

        // Mark the address as in use
        sqlx::query_as!(
            Address,
            r#"
            UPDATE addresses
            SET status = $1, instance_id = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
            address_status::IN_USE,
            instance_id,
            address.id
        )
        .fetch_one(pool)
        .await
        .map_err(Into::into)
    }

    /// Releases an IP address when an instance is deleted or detached.
    pub async fn release_ip(pool: &Pool<Postgres>, address_id: Uuid) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE addresses
            SET status = $1, instance_id = NULL, updated_at = NOW()
            WHERE id = $2
            "#,
            address_status::RESERVED,
            address_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Releases all IP addresses for an instance.
    pub async fn release_instance_ips(
        pool: &Pool<Postgres>,
        instance_id: Uuid,
    ) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE addresses
            SET status = $1, instance_id = NULL, updated_at = NOW()
            WHERE instance_id = $2
            "#,
            address_status::RESERVED,
            instance_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cidr_valid() {
        let (addr, prefix) = parse_cidr("10.0.0.0/24").unwrap();
        assert_eq!(addr, Ipv4Addr::new(10, 0, 0, 0));
        assert_eq!(prefix, 24);
    }

    #[test]
    fn test_parse_cidr_invalid() {
        assert!(parse_cidr("invalid").is_err());
        assert!(parse_cidr("10.0.0.0/33").is_err());
        assert!(parse_cidr("10.0.0.0").is_err());
    }

    #[test]
    fn test_calculate_gateway() {
        let gateway = calculate_gateway(&Ipv4Addr::new(10, 0, 0, 0), 24).unwrap();
        assert_eq!(gateway, Ipv4Addr::new(10, 0, 0, 1));
    }
}
