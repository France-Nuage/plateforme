//! Security Group management.
//!
//! Provides the SecurityGroup and SecurityRule data models and services
//! for managing network access control with Zero Trust (DENY ALL by default).

use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::network::{VPC, VPCFactory};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display as StrumDisplay, EnumString};
use uuid::Uuid;

/// Traffic direction for a security rule.
#[derive(Clone, Debug, Default, StrumDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    #[default]
    Inbound,
    Outbound,
}

impl From<Direction> for String {
    fn from(value: Direction) -> Self {
        value.to_string()
    }
}

impl From<String> for Direction {
    fn from(value: String) -> Self {
        Direction::from_str(&value).unwrap_or_default()
    }
}

/// Network protocol for a security rule.
#[derive(Clone, Debug, Default, StrumDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    #[default]
    All,
}

impl From<Protocol> for String {
    fn from(value: Protocol) -> Self {
        value.to_string()
    }
}

impl From<String> for Protocol {
    fn from(value: String) -> Self {
        Protocol::from_str(&value).unwrap_or_default()
    }
}

/// Action to take when a rule matches.
#[derive(Clone, Debug, Default, StrumDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    Allow,
    #[default]
    Deny,
}

impl From<Action> for String {
    fn from(value: Action) -> Self {
        value.to_string()
    }
}

impl From<String> for Action {
    fn from(value: String) -> Self {
        Action::from_str(&value).unwrap_or_default()
    }
}

/// Security Group represents a set of firewall rules.
#[derive(Clone, Debug, Default, Factory, Persistable, Resource)]
pub struct SecurityGroup {
    /// Unique identifier
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// ID of the parent VPC
    #[fabrique(relation = "VPC", referenced_key = "id")]
    pub vpc_id: Uuid,
    /// Human-readable name (unique within VPC)
    pub name: String,
    /// Description of the security group's purpose
    pub description: Option<String>,
    /// Whether this is the default security group for the VPC
    pub is_default: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl SecurityGroup {
    /// Find a security group by ID.
    pub async fn find_one_by_id(
        pool: &Pool<Postgres>,
        id: Uuid,
    ) -> Result<SecurityGroup, sqlx::Error> {
        sqlx::query_as::<_, SecurityGroup>("SELECT * FROM security_groups WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Find all security groups for a VPC.
    pub async fn find_by_vpc_id(
        pool: &Pool<Postgres>,
        vpc_id: Uuid,
    ) -> Result<Vec<SecurityGroup>, sqlx::Error> {
        sqlx::query_as::<_, SecurityGroup>("SELECT * FROM security_groups WHERE vpc_id = $1")
            .bind(vpc_id)
            .fetch_all(pool)
            .await
    }

    /// Find the default security group for a VPC.
    pub async fn find_default_for_vpc(
        pool: &Pool<Postgres>,
        vpc_id: Uuid,
    ) -> Result<Option<SecurityGroup>, sqlx::Error> {
        sqlx::query_as::<_, SecurityGroup>(
            "SELECT * FROM security_groups WHERE vpc_id = $1 AND is_default = true",
        )
        .bind(vpc_id)
        .fetch_optional(pool)
        .await
    }

    /// Find a security group by name within a VPC.
    pub async fn find_by_name(
        pool: &Pool<Postgres>,
        vpc_id: Uuid,
        name: &str,
    ) -> Result<Option<SecurityGroup>, sqlx::Error> {
        sqlx::query_as::<_, SecurityGroup>(
            "SELECT * FROM security_groups WHERE vpc_id = $1 AND name = $2",
        )
        .bind(vpc_id)
        .bind(name)
        .fetch_optional(pool)
        .await
    }
}

/// Security Rule represents a single firewall rule.
#[derive(Clone, Debug, Default, Factory, Persistable, Resource)]
pub struct SecurityRule {
    /// Unique identifier
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// ID of the parent security group
    #[fabrique(relation = "SecurityGroup", referenced_key = "id")]
    pub security_group_id: Uuid,
    /// Traffic direction
    #[fabrique(as = "String")]
    pub direction: Direction,
    /// Network protocol
    #[fabrique(as = "String")]
    pub protocol: Protocol,
    /// Start of port range
    pub port_from: Option<i32>,
    /// End of port range
    pub port_to: Option<i32>,
    /// Source/destination CIDR
    pub source_cidr: String,
    /// Action to take
    #[fabrique(as = "String")]
    pub action: Action,
    /// Rule priority (lower = higher priority)
    pub priority: i32,
    /// Description of the rule
    pub description: Option<String>,
}

impl SecurityRule {
    /// Find a security rule by ID.
    pub async fn find_one_by_id(
        pool: &Pool<Postgres>,
        id: Uuid,
    ) -> Result<SecurityRule, sqlx::Error> {
        sqlx::query_as::<_, SecurityRule>("SELECT * FROM security_rules WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Find all rules for a security group.
    pub async fn find_by_security_group_id(
        pool: &Pool<Postgres>,
        security_group_id: Uuid,
    ) -> Result<Vec<SecurityRule>, sqlx::Error> {
        sqlx::query_as::<_, SecurityRule>(
            "SELECT * FROM security_rules WHERE security_group_id = $1 ORDER BY priority",
        )
        .bind(security_group_id)
        .fetch_all(pool)
        .await
    }
}

/// Request to create a new security group.
#[derive(Clone, Debug)]
pub struct SecurityGroupCreateRequest {
    /// ID of the parent VPC
    pub vpc_id: Uuid,
    /// Name for the security group
    pub name: String,
    /// Description (optional)
    pub description: Option<String>,
}

/// Request to update a security group.
#[derive(Clone, Debug)]
pub struct SecurityGroupUpdateRequest {
    /// ID of the security group
    pub id: Uuid,
    /// New name (optional)
    pub name: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
}

/// Request to add a firewall rule.
#[derive(Clone, Debug)]
pub struct SecurityRuleCreateRequest {
    /// ID of the security group
    pub security_group_id: Uuid,
    /// Traffic direction
    pub direction: Direction,
    /// Network protocol
    pub protocol: Protocol,
    /// Start of port range
    pub port_from: Option<i32>,
    /// End of port range
    pub port_to: Option<i32>,
    /// Source/destination CIDR
    pub source_cidr: String,
    /// Action to take
    pub action: Action,
    /// Rule priority (1-65535)
    pub priority: i32,
    /// Description of the rule
    pub description: Option<String>,
}

/// Service for managing security groups.
#[derive(Clone, Debug)]
pub struct SecurityGroups<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> SecurityGroups<A> {
    /// Creates a new SecurityGroups service.
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    /// Lists all security groups in a VPC.
    pub async fn list<P: Principal + Sync>(
        &mut self,
        principal: &P,
        vpc_id: Uuid,
    ) -> Result<Vec<SecurityGroup>, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<VPC>(&vpc_id)
            .await?;

        SecurityGroup::find_by_vpc_id(&self.db, vpc_id)
            .await
            .map_err(Into::into)
    }

    /// Gets a security group by ID with its rules.
    pub async fn get<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(SecurityGroup, Vec<SecurityRule>), Error> {
        let sg = SecurityGroup::find_one_by_id(&self.db, id).await?;

        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        let rules = SecurityRule::find_by_security_group_id(&self.db, id).await?;

        Ok((sg, rules))
    }

    /// Creates a new security group.
    pub async fn create<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: SecurityGroupCreateRequest,
    ) -> Result<SecurityGroup, Error> {
        self.auth
            .can(principal)
            .perform(Permission::CreateSecurityGroup)
            .over::<VPC>(&request.vpc_id)
            .await?;

        // Check name uniqueness within VPC
        if SecurityGroup::find_by_name(&self.db, request.vpc_id, &request.name)
            .await?
            .is_some()
        {
            return Err(Error::Other(format!(
                "Security group '{}' already exists in this VPC",
                request.name
            )));
        }

        let sg = SecurityGroup::factory()
            .id(Uuid::new_v4())
            .vpc_id(request.vpc_id)
            .name(request.name)
            .description(request.description)
            .is_default(false)
            .create(&self.db)
            .await?;

        // Create authorization relationship
        let vpc = VPC::some(request.vpc_id);
        Relationship::new(&vpc, Relation::Parent, &sg)
            .publish(&self.db)
            .await?;

        Ok(sg)
    }

    /// Updates a security group.
    pub async fn update<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: SecurityGroupUpdateRequest,
    ) -> Result<SecurityGroup, Error> {
        let sg = SecurityGroup::find_one_by_id(&self.db, request.id).await?;

        self.auth
            .can(principal)
            .perform(Permission::UpdateSecurityGroup)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        // Cannot rename default security group
        if sg.is_default && request.name.is_some() {
            return Err(Error::Other(
                "Cannot rename default security group".to_string(),
            ));
        }

        // Build update query with parameterized values to prevent SQL injection
        let mut set_clauses = Vec::new();
        let mut param_index = 2; // $1 is reserved for id

        if let Some(ref name) = request.name {
            // Check uniqueness
            if let Some(existing) = SecurityGroup::find_by_name(&self.db, sg.vpc_id, name).await? {
                if existing.id != request.id {
                    return Err(Error::Other(format!(
                        "Security group '{}' already exists in this VPC",
                        name
                    )));
                }
            }
            set_clauses.push(format!("name = ${}", param_index));
            param_index += 1;
        }
        if request.description.is_some() {
            set_clauses.push(format!("description = ${}", param_index));
            // param_index += 1; // Uncomment if more fields are added
        }

        if set_clauses.is_empty() {
            return Ok(sg);
        }

        set_clauses.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE security_groups SET {} WHERE id = $1",
            set_clauses.join(", ")
        );

        let mut query_builder = sqlx::query(&query).bind(request.id);
        if let Some(ref name) = request.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(ref description) = request.description {
            query_builder = query_builder.bind(description);
        }

        query_builder.execute(&self.db).await?;

        SecurityGroup::find_one_by_id(&self.db, request.id)
            .await
            .map_err(Into::into)
    }

    /// Deletes a security group.
    pub async fn delete<P: Principal + Sync>(
        &mut self,
        principal: &P,
        id: Uuid,
    ) -> Result<(), Error> {
        let sg = SecurityGroup::find_one_by_id(&self.db, id).await?;

        self.auth
            .can(principal)
            .perform(Permission::Delete)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        // Cannot delete default security group
        if sg.is_default {
            return Err(Error::Other(
                "Cannot delete default security group".to_string(),
            ));
        }

        // Check if attached to any interfaces
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sg_interface_associations WHERE security_group_id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        if count.0 > 0 {
            return Err(Error::Other(
                "Cannot delete security group attached to interfaces".to_string(),
            ));
        }

        // Delete rules first
        sqlx::query("DELETE FROM security_rules WHERE security_group_id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        // Delete security group
        sqlx::query("DELETE FROM security_groups WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Adds a rule to a security group.
    pub async fn add_rule<P: Principal + Sync>(
        &mut self,
        principal: &P,
        request: SecurityRuleCreateRequest,
    ) -> Result<SecurityRule, Error> {
        let sg = SecurityGroup::find_one_by_id(&self.db, request.security_group_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::UpdateSecurityGroup)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        // Validate priority
        if request.priority < 1 || request.priority > 65535 {
            return Err(Error::Other(
                "Priority must be between 1 and 65535".to_string(),
            ));
        }

        // Validate port range for TCP/UDP
        if (request.protocol == Protocol::Tcp || request.protocol == Protocol::Udp)
            && (request.port_from.is_none() || request.port_to.is_none())
        {
            return Err(Error::Other(
                "Port range required for TCP/UDP protocols".to_string(),
            ));
        }

        let rule = SecurityRule::factory()
            .id(Uuid::new_v4())
            .security_group_id(request.security_group_id)
            .direction(request.direction)
            .protocol(request.protocol)
            .port_from(request.port_from)
            .port_to(request.port_to)
            .source_cidr(request.source_cidr)
            .action(request.action)
            .priority(request.priority)
            .description(request.description)
            .create(&self.db)
            .await?;

        // Update security group timestamp
        sqlx::query("UPDATE security_groups SET updated_at = NOW() WHERE id = $1")
            .bind(request.security_group_id)
            .execute(&self.db)
            .await?;

        Ok(rule)
    }

    /// Removes a rule from a security group.
    pub async fn remove_rule<P: Principal + Sync>(
        &mut self,
        principal: &P,
        rule_id: Uuid,
    ) -> Result<(), Error> {
        let rule = SecurityRule::find_one_by_id(&self.db, rule_id).await?;
        let sg = SecurityGroup::find_one_by_id(&self.db, rule.security_group_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::UpdateSecurityGroup)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        sqlx::query("DELETE FROM security_rules WHERE id = $1")
            .bind(rule_id)
            .execute(&self.db)
            .await?;

        // Update security group timestamp
        sqlx::query("UPDATE security_groups SET updated_at = NOW() WHERE id = $1")
            .bind(sg.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Attaches a security group to an instance interface.
    pub async fn attach_to_interface<P: Principal + Sync>(
        &mut self,
        principal: &P,
        security_group_id: Uuid,
        interface_id: Uuid,
    ) -> Result<(), Error> {
        let sg = SecurityGroup::find_one_by_id(&self.db, security_group_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::UpdateSecurityGroup)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        // Insert association (ignore if already exists)
        sqlx::query(
            r#"
            INSERT INTO sg_interface_associations (security_group_id, instance_interface_id)
            VALUES ($1, $2)
            ON CONFLICT (security_group_id, instance_interface_id) DO NOTHING
            "#,
        )
        .bind(security_group_id)
        .bind(interface_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Detaches a security group from an instance interface.
    pub async fn detach_from_interface<P: Principal + Sync>(
        &mut self,
        principal: &P,
        security_group_id: Uuid,
        interface_id: Uuid,
    ) -> Result<(), Error> {
        let sg = SecurityGroup::find_one_by_id(&self.db, security_group_id).await?;

        self.auth
            .can(principal)
            .perform(Permission::UpdateSecurityGroup)
            .over::<VPC>(&sg.vpc_id)
            .await?;

        sqlx::query(
            "DELETE FROM sg_interface_associations WHERE security_group_id = $1 AND instance_interface_id = $2"
        )
        .bind(security_group_id)
        .bind(interface_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
