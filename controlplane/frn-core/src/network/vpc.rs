//! VPC (Virtual Private Cloud) management.
//!
//! Provides the VPC data model and VPCs service for creating, managing,
//! and controlling isolated network environments with authorization checks.

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::network::SecurityGroup;
use crate::resourcemanager::{Organization, OrganizationFactory};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display as StrumDisplay, EnumString};
use uuid::Uuid;

/// State of a VPC in its lifecycle.
#[derive(Clone, Debug, Default, StrumDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum VPCState {
    #[default]
    Pending,
    Creating,
    Active,
    Error,
    Deleting,
}

impl From<VPCState> for String {
    fn from(value: VPCState) -> Self {
        value.to_string()
    }
}

impl From<String> for VPCState {
    fn from(value: String) -> Self {
        VPCState::from_str(&value).unwrap_or_default()
    }
}

/// VPC represents a Virtual Private Cloud - an isolated network environment.
#[derive(Clone, Debug, Factory, Persistable, Resource)]
pub struct VPC {
    /// Unique identifier for the VPC
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// Human-readable name of the VPC
    pub name: String,
    /// URL-friendly slug, unique worldwide
    pub slug: String,
    /// ID of the organization this VPC belongs to
    #[fabrique(relation = "Organization", referenced_key = "id")]
    pub organization_id: Uuid,
    /// Region where the VPC is deployed (e.g., fr-paris-1)
    pub region: String,
    /// Proxmox SDN zone identifier
    pub sdn_zone_id: String,
    /// VXLAN tag for network isolation (1-16777215)
    pub vxlan_tag: i32,
    /// Current state of the VPC
    #[fabrique(as = "String")]
    pub state: VPCState,
    /// Maximum Transmission Unit (1280-1500, default 1450 for VXLAN)
    pub mtu: i32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Default for VPC {
    fn default() -> Self {
        Self {
            id: Uuid::default(),
            name: String::default(),
            slug: String::default(),
            organization_id: Uuid::default(),
            region: String::default(),
            sdn_zone_id: String::default(),
            vxlan_tag: 1, // Minimum valid value per DB constraint (>= 1)
            state: VPCState::default(),
            mtu: 1450, // Default for VXLAN, within DB constraint (1280-1500)
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        }
    }
}

impl VPC {
    /// Find a VPC by its ID.
    pub async fn find_one_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<VPC, sqlx::Error> {
        sqlx::query_as::<_, VPC>("SELECT * FROM vpcs WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Find a VPC by its slug.
    pub async fn find_one_by_slug(
        pool: &Pool<Postgres>,
        slug: &str,
    ) -> Result<Option<VPC>, sqlx::Error> {
        sqlx::query_as::<_, VPC>("SELECT * FROM vpcs WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    /// Find all VPCs for an organization.
    pub async fn find_by_organization_id(
        pool: &Pool<Postgres>,
        organization_id: Uuid,
    ) -> Result<Vec<VPC>, sqlx::Error> {
        sqlx::query_as::<_, VPC>("SELECT * FROM vpcs WHERE organization_id = $1")
            .bind(organization_id)
            .fetch_all(pool)
            .await
    }
}

/// Request to create a new VPC.
#[derive(Clone, Debug)]
pub struct VPCCreateRequest {
    /// Human-readable name for the VPC
    pub name: String,
    /// URL-friendly slug (must be globally unique)
    pub slug: String,
    /// ID of the organization to create the VPC in
    pub organization_id: Uuid,
    /// Region for the VPC (optional, defaults to fr-paris-1)
    pub region: Option<String>,
    /// MTU setting (optional, default 1450)
    pub mtu: Option<i32>,
}

/// Request to update an existing VPC.
#[derive(Clone, Debug)]
pub struct VPCUpdateRequest {
    /// ID of the VPC to update
    pub id: Uuid,
    /// New name (optional)
    pub name: Option<String>,
    /// New MTU setting (optional)
    pub mtu: Option<i32>,
}

/// Service for managing VPCs.
#[derive(Clone, Debug)]
pub struct VPCs<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> VPCs<A> {
    /// Creates a new VPCs service.
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Lists all VPCs accessible to the principal.
    pub async fn list<P: Principal>(
        &mut self,
        principal: &P,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<VPC>, Error> {
        // Use lookup to find VPCs the principal has access to
        let vpcs = self
            .auth
            .lookup::<VPC>()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await?;

        // Filter by organization if specified
        match organization_id {
            Some(org_id) => Ok(vpcs
                .into_iter()
                .filter(|v| v.organization_id == org_id)
                .collect()),
            None => Ok(vpcs),
        }
    }

    /// Gets a VPC by ID.
    pub async fn get<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<VPC, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<VPC>(&id)
            .await?;

        VPC::find_one_by_id(&self.db, id).await.map_err(Into::into)
    }

    /// Creates a new VPC.
    pub async fn create<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: VPCCreateRequest,
    ) -> Result<VPC, Error> {
        // Check permission to create VPC in the organization
        self.auth
            .can(principal)
            .perform(Permission::CreateVPC)
            .over::<Organization>(&request.organization_id)
            .await?;

        // Check slug uniqueness
        let existing = VPC::find_one_by_slug(&self.db, &request.slug).await?;
        if existing.is_some() {
            return Err(Error::SlugAlreadyExists(request.slug));
        }

        // Allocate VXLAN tag from sequence
        let vxlan_tag: (i32,) = sqlx::query_as("SELECT nextval('vxlan_tag_seq')::int4")
            .fetch_one(&self.db)
            .await?;

        // Generate SDN zone ID
        let sdn_zone_id = format!("zone-{}", &request.slug);

        // Default region and MTU
        let region = request.region.unwrap_or_else(|| "fr-paris-1".to_string());
        let mtu = request.mtu.unwrap_or(1450);

        // Create VPC with state = CREATING
        let vpc = VPC::factory()
            .id(Uuid::new_v4())
            .name(request.name)
            .slug(request.slug)
            .organization_id(request.organization_id)
            .region(region)
            .sdn_zone_id(sdn_zone_id)
            .vxlan_tag(vxlan_tag.0)
            .state(VPCState::Creating)
            .mtu(mtu)
            .create(&self.db)
            .await?;

        // Create authorization relationship
        Relationship::new(
            &Organization::some(request.organization_id),
            Relation::Parent,
            &vpc,
        )
        .publish(&self.db)
        .await?;

        // Create default security group with DENY ALL rules
        self.create_default_security_group(&vpc).await?;

        // Update state to ACTIVE (Proxmox SDN integration will be done in Phase 5)
        sqlx::query("UPDATE vpcs SET state = $1, updated_at = NOW() WHERE id = $2")
            .bind(VPCState::Active.to_string())
            .bind(vpc.id)
            .execute(&self.db)
            .await?;

        // Fetch updated VPC
        VPC::find_one_by_id(&self.db, vpc.id)
            .await
            .map_err(Into::into)
    }

    /// Creates the default security group for a VPC with DENY ALL rules.
    async fn create_default_security_group(&self, vpc: &VPC) -> Result<SecurityGroup, Error> {
        let sg_id = Uuid::new_v4();

        // Create the default security group
        sqlx::query(
            r#"
            INSERT INTO security_groups (id, vpc_id, name, description, is_default)
            VALUES ($1, $2, 'default', 'Default security group - DENY ALL', true)
            "#,
        )
        .bind(sg_id)
        .bind(vpc.id)
        .execute(&self.db)
        .await?;

        // Create DENY ALL INBOUND rule
        sqlx::query(
            r#"
            INSERT INTO security_rules (id, security_group_id, direction, protocol, source_cidr, action, priority, description)
            VALUES ($1, $2, 'INBOUND', 'ALL', '0.0.0.0/0', 'DENY', 65535, 'Default deny all inbound')
            "#
        )
        .bind(Uuid::new_v4())
        .bind(sg_id)
        .execute(&self.db)
        .await?;

        // Create DENY ALL OUTBOUND rule
        sqlx::query(
            r#"
            INSERT INTO security_rules (id, security_group_id, direction, protocol, source_cidr, action, priority, description)
            VALUES ($1, $2, 'OUTBOUND', 'ALL', '0.0.0.0/0', 'DENY', 65535, 'Default deny all outbound')
            "#
        )
        .bind(Uuid::new_v4())
        .bind(sg_id)
        .execute(&self.db)
        .await?;

        // Fetch and return the created security group
        SecurityGroup::find_one_by_id(&self.db, sg_id)
            .await
            .map_err(Into::into)
    }

    /// Updates an existing VPC.
    pub async fn update<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: VPCUpdateRequest,
    ) -> Result<VPC, Error> {
        self.auth
            .can(principal)
            .perform(Permission::UpdateVPC)
            .over::<VPC>(&request.id)
            .await?;

        // Build update query with parameterized values to prevent SQL injection
        let mut set_clauses = Vec::new();
        let mut param_index = 2; // $1 is reserved for id

        if request.name.is_some() {
            set_clauses.push(format!("name = ${}", param_index));
            param_index += 1;
        }
        if request.mtu.is_some() {
            set_clauses.push(format!("mtu = ${}", param_index));
            // param_index += 1; // Uncomment if more fields are added
        }

        if set_clauses.is_empty() {
            return VPC::find_one_by_id(&self.db, request.id)
                .await
                .map_err(Into::into);
        }

        set_clauses.push("updated_at = NOW()".to_string());
        let query = format!("UPDATE vpcs SET {} WHERE id = $1", set_clauses.join(", "));

        let mut query_builder = sqlx::query(&query).bind(request.id);
        if let Some(ref name) = request.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(mtu) = request.mtu {
            query_builder = query_builder.bind(mtu);
        }

        query_builder.execute(&self.db).await?;

        VPC::find_one_by_id(&self.db, request.id)
            .await
            .map_err(Into::into)
    }

    /// Deletes a VPC.
    pub async fn delete<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over::<VPC>(&id)
            .await?;

        // Check if VPC exists
        let _vpc = VPC::find_one_by_id(&self.db, id).await?;

        // Check for existing VNets
        let vnet_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vnets WHERE vpc_id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;

        if vnet_count.0 > 0 {
            return Err(Error::Other(
                "Cannot delete VPC with existing VNets".to_string(),
            ));
        }

        // Update state to DELETING
        sqlx::query("UPDATE vpcs SET state = $1, updated_at = NOW() WHERE id = $2")
            .bind(VPCState::Deleting.to_string())
            .bind(id)
            .execute(&self.db)
            .await?;

        // Delete security groups and rules
        sqlx::query("DELETE FROM security_rules WHERE security_group_id IN (SELECT id FROM security_groups WHERE vpc_id = $1)")
            .bind(id)
            .execute(&self.db)
            .await?;
        sqlx::query("DELETE FROM security_groups WHERE vpc_id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        // Delete VPC (Proxmox SDN cleanup will be done in Phase 5)
        sqlx::query("DELETE FROM vpcs WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
