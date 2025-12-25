use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::identity::{ServiceAccount, User};
use crate::operations::{Operation, OperationType};
use crate::resourcemanager::{DEFAULT_PROJECT_NAME, Project};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// Represents a user's membership in an organization.
#[derive(Debug)]
pub struct OrganizationMember {
    /// The user's ID
    pub user_id: Uuid,
    /// The organization ID
    pub organization_id: Uuid,
    /// The user's email
    pub email: String,
    /// The user's name
    pub name: String,
    /// Optional role ID assigned to the member
    pub role_id: Option<Uuid>,
    /// When the user joined the organization
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Default, Factory, Persistable, Resource)]
pub struct Organization {
    /// The organization id
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The organization name
    pub name: String,
    /// The organization slug (DNS-compatible identifier)
    pub slug: String,
    /// The organization parent, if any
    pub parent_id: Option<Uuid>,
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Generate a DNS-compatible slug from a name.
///
/// The slug follows RFC 1123 subdomain rules:
/// - Only lowercase alphanumeric characters and hyphens
/// - Cannot start or end with a hyphen
/// - Maximum 63 characters
fn generate_slug(name: &str) -> String {
    slug::slugify(name)
        .chars()
        .take(63)
        .collect::<String>()
        .trim_end_matches('-')
        .to_string()
}

#[derive(Clone)]
pub struct Organizations<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Organizations<A> {
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    pub async fn list<P: Principal>(&mut self, principal: &P) -> Result<Vec<Organization>, Error> {
        self.auth
            .lookup::<Organization>()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await
    }

    pub async fn create_organization<P: Principal + Sync>(
        &mut self,
        connection: &Pool<Postgres>,
        _principal: &P,
        name: String,
        parent_id: Option<Uuid>,
    ) -> Result<Organization, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&Organization::any())
        //     .await?;

        tracing::info!(
            "received request to create organization with name '{}' and parent id '{:?}'",
            &name,
            &parent_id
        );

        // Generate slug from name
        let slug = generate_slug(&name);

        // Check for slug collision
        let existing: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM organizations WHERE slug = $1")
                .bind(&slug)
                .fetch_optional(connection)
                .await?;

        if existing.is_some() {
            return Err(Error::SlugAlreadyExists(slug));
        }

        // Create the organization
        let organization = Organization::factory()
            .id(Uuid::new_v4())
            .name(name)
            .slug(slug)
            .parent_id(parent_id)
            .create(connection)
            .await?;

        // Create the parent relationship if specified
        if let Some(parent_id) = parent_id {
            let parent: Organization = sqlx::query_as!(
                Organization,
                "SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE id = $1",
                parent_id
            )
            .fetch_one(&self.db)
            .await?;

            Relationship::new(&parent, Relation::Parent, &organization)
                .publish(&self.db)
                .await?;
        }

        let project = Project::factory()
            .id(Uuid::new_v4())
            .name(DEFAULT_PROJECT_NAME.to_owned())
            .organization_id(organization.id)
            .create(&self.db)
            .await?;

        Relationship::new(&organization, Relation::Parent, &project)
            .publish(&self.db)
            .await?;

        Ok(organization)
    }

    pub async fn add_service_account(
        &mut self,
        organization: &Organization,
        service_account: &ServiceAccount,
    ) -> Result<(), Error> {
        // Create the associated in the relational database
        sqlx::query!("INSERT INTO organization_service_account(organization_id, service_account_id) VALUES ($1, $2) ON CONFLICT (organization_id, service_account_id) DO NOTHING", organization.id(), service_account.id()).execute(&self.db).await?;

        // Create the relation for dispatch in the authorization database
        Relationship::new(service_account, Relation::Member, organization)
            .publish(&self.db)
            .await?;

        Ok(())
    }

    pub async fn add_user(
        &mut self,
        organization: &Organization,
        user: &User,
    ) -> Result<(), Error> {
        // Create the associated in the relational database
        sqlx::query!("INSERT INTO organization_user(organization_id, user_id) VALUES ($1, $2) ON CONFLICT (organization_id, user_id) DO NOTHING", organization.id(), user.id()).execute(&self.db).await?;

        // Create the relation for dispatch in the authorization database
        Relationship::new(user, Relation::Member, organization)
            .publish(&self.db)
            .await?;

        Ok(())
    }

    /// Lists all members of an organization.
    ///
    /// Returns users who are members of the organization.
    pub async fn list_members<P: Principal + Sync>(
        &self,
        principal: &P,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationMember>, Error> {
        // Check authorization - user must have Get permission on the organization
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over::<Organization>(&organization_id)
            .await?;

        // Fetch members with user details
        let rows = sqlx::query(
            r#"
            SELECT u.id as user_id, u.email, u.name, ou.organization_id, ou.created_at as joined_at
            FROM organization_user ou
            JOIN users u ON ou.user_id = u.id
            WHERE ou.organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.db)
        .await?;

        let members = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                OrganizationMember {
                    user_id: row.get("user_id"),
                    organization_id: row.get("organization_id"),
                    email: row.get("email"),
                    name: row.get("name"),
                    role_id: None, // Role support can be added later
                    joined_at: row.get("joined_at"),
                }
            })
            .collect();

        Ok(members)
    }

    /// Removes a user from an organization.
    ///
    /// This method:
    /// 1. Removes the user from the organization in the local database
    /// 2. Creates a SpiceDB operation to delete the membership relationship
    /// 3. Creates a Pangolin operation to remove the user from the organization
    ///
    /// # Arguments
    /// * `principal` - The principal performing the removal (must have RemoveMember permission)
    /// * `organization` - The organization to remove the user from
    /// * `user` - The user to remove
    ///
    /// # Returns
    /// A vector of Operations to be processed by the operations-worker:
    /// - SpiceDB DeleteRelationship operation
    /// - Pangolin RemoveUser operation
    pub async fn remove_user<P: Principal + Sync>(
        &mut self,
        principal: &P,
        organization: &Organization,
        user: &User,
    ) -> Result<Vec<Operation>, Error> {
        // Check authorization
        self.auth
            .can(principal)
            .perform(Permission::RemoveMember)
            .over::<Organization>(organization.id())
            .await?;

        // Remove from the local database
        sqlx::query("DELETE FROM organization_user WHERE organization_id = $1 AND user_id = $2")
            .bind(organization.id())
            .bind(user.id())
            .execute(&self.db)
            .await?;

        let mut operations = Vec::new();

        // Create SpiceDB operation to delete the membership relationship
        let spicedb_operation = Operation::new(
            OperationType::SpiceDbDeleteRelationship,
            "User",
            *user.id(),
            serde_json::json!({
                "subject_type": "User",
                "subject_id": user.id().to_string(),
                "relation": "Member",
                "object_type": "Organization",
                "object_id": organization.id().to_string()
            }),
        )
        .create(&self.db)
        .await?;
        operations.push(spicedb_operation);

        // Create Pangolin operation to remove the user from the organization
        let pangolin_operation = Operation::new(
            OperationType::PangolinRemoveUser,
            "User",
            *user.id(),
            serde_json::json!({
                "org_id": organization.slug,
                "user_id": user.id().to_string(),
                "email": user.email
            }),
        )
        .create(&self.db)
        .await?;
        operations.push(pangolin_operation);

        tracing::info!(
            organization_id = %organization.id(),
            user_id = %user.id(),
            user_email = %user.email,
            "user removed from organization, operations created for sync"
        );

        Ok(operations)
    }

    pub async fn initialize_root_organization(
        &mut self,
        organization_name: String,
    ) -> Result<Organization, Error> {
        // Attempt to retrieve the organization from the database
        let maybe_organization: Option<Organization> = sqlx::query_as!(
            Organization,
            "SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE name = $1 LIMIT 1",
            &organization_name
        )
        .fetch_optional(&self.db)
        .await?;

        // Create the root organization if there is no database match
        let organization = match maybe_organization {
            Some(organization) => organization,
            None => {
                let slug = generate_slug(&organization_name);

                // Check for slug collision
                let existing: Option<(Uuid,)> =
                    sqlx::query_as("SELECT id FROM organizations WHERE slug = $1")
                        .bind(&slug)
                        .fetch_optional(&self.db)
                        .await?;

                if existing.is_some() {
                    return Err(Error::SlugAlreadyExists(slug));
                }

                Organization::factory()
                    .name(organization_name)
                    .slug(slug)
                    .create(&self.db)
                    .await?
            }
        };

        // Create the default project for the root organization
        sqlx::query!(
            r#"
            INSERT INTO projects (name, organization_id)
            SELECT 'unattributed', $1
            WHERE NOT EXISTS (
                SELECT 1 FROM projects
                WHERE name = 'unattributed' AND organization_id = $1
            )
            "#,
            &organization.id
        )
        .execute(&self.db)
        .await?;

        Ok(organization)
    }
}
