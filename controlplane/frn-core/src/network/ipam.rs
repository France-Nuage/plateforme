//! IPAM (IP Address Management) service.
//!
//! Provides IP address allocation, release, and MAC address generation
//! for the France Nuage platform. Uses OUI prefix BC:24:11 for MAC addresses.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Resource};
use crate::network::{VPC, VNet, VNetFactory};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use ipnetwork::IpNetwork;
use rand::{Rng, SeedableRng};
use sqlx::{Pool, Postgres};
use std::net::IpAddr;
use std::str::FromStr;
use strum_macros::{Display as StrumDisplay, EnumString};
use uuid::Uuid;

/// France Nuage OUI prefix for MAC addresses.
pub const MAC_OUI_PREFIX: &str = "BC:24:11";

/// Type of IP allocation.
#[derive(Clone, Debug, Default, StrumDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AllocationType {
    #[default]
    Static,
    Dynamic,
    Reserved,
    Gateway,
}

impl From<AllocationType> for String {
    fn from(value: AllocationType) -> Self {
        value.to_string()
    }
}

impl From<String> for AllocationType {
    fn from(value: String) -> Self {
        AllocationType::from_str(&value).unwrap_or_default()
    }
}

/// IP address allocation record.
#[derive(Clone, Debug, Default, Factory, Persistable, Resource)]
pub struct IPAllocation {
    /// Unique identifier for the allocation
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// ID of the VNet this IP belongs to
    #[fabrique(relation = "VNet", referenced_key = "id")]
    pub vnet_id: Uuid,
    /// The allocated IP address
    pub address: String,
    /// Associated MAC address
    pub mac_address: Option<String>,
    /// ID of the instance interface using this IP (if any)
    pub instance_interface_id: Option<Uuid>,
    /// Type of allocation
    #[fabrique(as = "String")]
    pub allocation_type: AllocationType,
    /// Hostname for reverse DNS
    pub hostname: Option<String>,
    /// When the IP was allocated
    pub allocated_at: Option<DateTime<Utc>>,
    /// When the IP was released
    pub released_at: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl IPAllocation {
    /// Find an IP allocation by ID.
    pub async fn find_one_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<IPAllocation, sqlx::Error> {
        sqlx::query_as::<_, IPAllocation>("SELECT * FROM ip_addresses WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Find all IP allocations for a VNet.
    pub async fn find_by_vnet_id(pool: &Pool<Postgres>, vnet_id: Uuid) -> Result<Vec<IPAllocation>, sqlx::Error> {
        sqlx::query_as::<_, IPAllocation>("SELECT * FROM ip_addresses WHERE vnet_id = $1")
            .bind(vnet_id)
            .fetch_all(pool)
            .await
    }

    /// Check if an IP address is already allocated in a VNet.
    pub async fn is_allocated(pool: &Pool<Postgres>, vnet_id: Uuid, address: &str) -> Result<bool, sqlx::Error> {
        let result: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM ip_addresses WHERE vnet_id = $1 AND address = $2::inet"
        )
            .bind(vnet_id)
            .bind(address)
            .fetch_optional(pool)
            .await?;
        Ok(result.is_some())
    }

    /// Check if a MAC address is already in use.
    pub async fn is_mac_used(pool: &Pool<Postgres>, mac_address: &str) -> Result<bool, sqlx::Error> {
        let result: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM ip_addresses WHERE mac_address = $1"
        )
            .bind(mac_address)
            .fetch_optional(pool)
            .await?;
        Ok(result.is_some())
    }
}

/// Request to allocate an IP address.
#[derive(Clone, Debug)]
pub struct AllocateIPRequest {
    /// ID of the VNet to allocate from
    pub vnet_id: Uuid,
    /// Specific IP to allocate (optional)
    pub requested_ip: Option<String>,
    /// Hostname for reverse DNS
    pub hostname: Option<String>,
}

/// Request to reserve a specific IP address.
#[derive(Clone, Debug)]
pub struct ReserveIPRequest {
    /// ID of the VNet containing the IP
    pub vnet_id: Uuid,
    /// Specific IP address to reserve
    pub address: String,
    /// Reason for reservation
    pub reason: Option<String>,
}

/// Statistics for IP allocations in a VNet.
#[derive(Clone, Debug)]
pub struct IPAllocationStats {
    /// Total number of allocations
    pub total_count: i32,
    /// Number of available IPs
    pub available_count: i32,
}

/// Service for managing IP addresses.
#[derive(Clone, Debug)]
pub struct IPAM<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> IPAM<A> {
    /// Creates a new IPAM service.
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Lists all IP allocations in a VNet.
    pub async fn list_allocations<P: Principal + Sync>(
        &mut self,
        principal: &P,
        vnet_id: Uuid,
        filter_type: Option<AllocationType>,
    ) -> Result<(Vec<IPAllocation>, IPAllocationStats), Error> {
        // Get VNet and check access to parent VPC
        let vnet = VNet::find_one_by_id(&self.db, vnet_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        let allocations = match filter_type {
            Some(alloc_type) => {
                sqlx::query_as::<_, IPAllocation>(
                    "SELECT * FROM ip_addresses WHERE vnet_id = $1 AND allocation_type = $2"
                )
                .bind(vnet_id)
                .bind(alloc_type.to_string())
                .fetch_all(&self.db)
                .await?
            }
            None => IPAllocation::find_by_vnet_id(&self.db, vnet_id).await?,
        };

        // Calculate stats
        let stats = self.calculate_stats(&vnet, allocations.len() as i32)?;

        Ok((allocations, stats))
    }

    /// Calculates allocation statistics for a VNet.
    fn calculate_stats(&self, vnet: &VNet, allocated_count: i32) -> Result<IPAllocationStats, Error> {
        let network: IpNetwork = vnet.subnet.parse()
            .map_err(|e| Error::Other(format!("Invalid subnet: {}", e)))?;

        // Total usable IPs (excluding network and broadcast)
        let total_ips = match network {
            IpNetwork::V4(net) => net.size() as i32 - 2,
            IpNetwork::V6(net) => {
                // For IPv6, limit to a reasonable number
                let size = net.size();
                if size > i32::MAX as u128 {
                    i32::MAX
                } else {
                    size as i32 - 2
                }
            }
        };

        Ok(IPAllocationStats {
            total_count: allocated_count,
            available_count: total_ips - allocated_count,
        })
    }

    /// Allocates an IP address from a VNet.
    pub async fn allocate<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: AllocateIPRequest,
    ) -> Result<IPAllocation, Error> {
        // Get VNet and check access to parent VPC
        let vnet = VNet::find_one_by_id(&self.db, request.vnet_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::AllocateIP)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        // Determine the IP to allocate
        let address = match request.requested_ip {
            Some(ip) => {
                // Validate requested IP is in subnet
                self.validate_ip_in_subnet(&ip, &vnet.subnet)?;

                // Check if already allocated
                if IPAllocation::is_allocated(&self.db, request.vnet_id, &ip).await? {
                    return Err(Error::Other(format!("IP address {} is already allocated", ip)));
                }
                ip
            }
            None => {
                // Find next available IP
                self.find_next_available_ip(&vnet).await?
            }
        };

        // Generate unique MAC address
        let mac_address = self.generate_unique_mac().await?;

        // Create allocation
        let alloc_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ip_addresses (id, vnet_id, address, mac_address, allocation_type, hostname, allocated_at)
            VALUES ($1, $2, $3::inet, $4, 'STATIC', $5, NOW())
            "#
        )
        .bind(alloc_id)
        .bind(request.vnet_id)
        .bind(&address)
        .bind(&mac_address)
        .bind(&request.hostname)
        .execute(&self.db)
        .await?;

        IPAllocation::find_one_by_id(&self.db, alloc_id).await.map_err(Into::into)
    }

    /// Validates that an IP address is within a subnet.
    fn validate_ip_in_subnet(&self, ip: &str, subnet: &str) -> Result<(), Error> {
        let network: IpNetwork = subnet.parse()
            .map_err(|e| Error::Other(format!("Invalid subnet: {}", e)))?;

        let addr: IpAddr = ip.parse()
            .map_err(|e| Error::Other(format!("Invalid IP address: {}", e)))?;

        if !network.contains(addr) {
            return Err(Error::Other(format!("IP {} is not within subnet {}", ip, subnet)));
        }

        // Check if it's the network or broadcast address
        if let IpNetwork::V4(net) = network {
            if let IpAddr::V4(v4) = addr {
                if v4 == net.network() || v4 == net.broadcast() {
                    return Err(Error::Other("Cannot allocate network or broadcast address".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Finds the next available IP address in a VNet.
    async fn find_next_available_ip(&self, vnet: &VNet) -> Result<String, Error> {
        let network: IpNetwork = vnet.subnet.parse()
            .map_err(|e| Error::Other(format!("Invalid subnet: {}", e)))?;

        // Get all allocated IPs
        let allocations = IPAllocation::find_by_vnet_id(&self.db, vnet.id).await?;
        let allocated_ips: std::collections::HashSet<String> = allocations
            .into_iter()
            .map(|a| a.address)
            .collect();

        match network {
            IpNetwork::V4(net) => {
                // Skip network address (first) and gateway (usually second)
                for ip in net.iter().skip(2) {
                    // Skip broadcast
                    if ip == net.broadcast() {
                        continue;
                    }
                    let ip_str = ip.to_string();
                    if !allocated_ips.contains(&ip_str) {
                        return Ok(ip_str);
                    }
                }
            }
            IpNetwork::V6(net) => {
                // For IPv6, start from the second address
                let mut count = 0;
                for ip in net.iter() {
                    count += 1;
                    if count <= 2 {
                        continue; // Skip network and gateway
                    }
                    if count > 65536 {
                        break; // Limit search
                    }
                    let ip_str = ip.to_string();
                    if !allocated_ips.contains(&ip_str) {
                        return Ok(ip_str);
                    }
                }
            }
        }

        Err(Error::Other("No available IP addresses in subnet".to_string()))
    }

    /// Generates a unique MAC address with France Nuage OUI prefix.
    async fn generate_unique_mac(&self) -> Result<String, Error> {
        let mut rng = rand::rngs::StdRng::from_entropy();

        for _ in 0..100 {
            // Generate random NIC portion
            let nic1: u8 = rng.r#gen();
            let nic2: u8 = rng.r#gen();
            let nic3: u8 = rng.r#gen();

            let mac = format!("{}:{:02X}:{:02X}:{:02X}", MAC_OUI_PREFIX, nic1, nic2, nic3);

            // Check uniqueness
            if !IPAllocation::is_mac_used(&self.db, &mac).await? {
                return Ok(mac);
            }
        }

        Err(Error::Other("Failed to generate unique MAC address after 100 attempts".to_string()))
    }

    /// Generates a MAC address without database check (for external use).
    pub async fn generate_mac<P: Principal + Sync>(&mut self, _principal: &P) -> Result<String, Error> {
        self.generate_unique_mac().await
    }

    /// Releases an allocated IP address.
    pub async fn release<P: Principal + Sync>(
        &mut self,
        principal: &P,
        allocation_id: Uuid,
    ) -> Result<String, Error> {
        let allocation = IPAllocation::find_one_by_id(&self.db, allocation_id).await?;
        let vnet = VNet::find_one_by_id(&self.db, allocation.vnet_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::ReleaseIP)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        // Cannot release gateway IPs
        if allocation.allocation_type == AllocationType::Gateway {
            return Err(Error::Other("Cannot release gateway IP address".to_string()));
        }

        let address = allocation.address.clone();

        // Delete the allocation
        sqlx::query("DELETE FROM ip_addresses WHERE id = $1")
            .bind(allocation_id)
            .execute(&self.db)
            .await?;

        Ok(address)
    }

    /// Reserves a specific IP address.
    pub async fn reserve<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: ReserveIPRequest,
    ) -> Result<IPAllocation, Error> {
        let vnet = VNet::find_one_by_id(&self.db, request.vnet_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::ReserveIP)
            .over::<VPC>(&vnet.vpc_id)
            .await?;

        // Validate IP is in subnet
        self.validate_ip_in_subnet(&request.address, &vnet.subnet)?;

        // Check if already allocated
        if IPAllocation::is_allocated(&self.db, request.vnet_id, &request.address).await? {
            return Err(Error::Other(format!("IP address {} is already allocated", request.address)));
        }

        // Create reservation
        let alloc_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ip_addresses (id, vnet_id, address, allocation_type, hostname, allocated_at)
            VALUES ($1, $2, $3::inet, 'RESERVED', $4, NOW())
            "#
        )
        .bind(alloc_id)
        .bind(request.vnet_id)
        .bind(&request.address)
        .bind(&request.reason)
        .execute(&self.db)
        .await?;

        IPAllocation::find_one_by_id(&self.db, alloc_id).await.map_err(Into::into)
    }
}
