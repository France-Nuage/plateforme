use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::identity::{ServiceAccount, User};
use crate::longrunning::Operation;
use crate::resourcemanager::{DEFAULT_PROJECT_NAME, Project};
use fabrique::{Factory, Model, Persist};
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, Resource)]
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
        let organization = Organization {
            id: Uuid::new_v4(),
            name,
            slug,
            parent_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
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

            Operation::write_relationships(vec![Relationship::new(
                &parent,
                Relation::Parent,
                &organization,
            )])
            .dispatch(&self.db)
            .await?;
        }

        let project = Project::factory()
            .id(Uuid::new_v4())
            .name(DEFAULT_PROJECT_NAME.to_owned())
            .organization_id(organization.id)
            .create(&self.db)
            .await?;

        Operation::write_relationships(vec![Relationship::new(
            &organization,
            Relation::Parent,
            &project,
        )])
        .dispatch(&self.db)
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
        Operation::write_relationships(vec![Relationship::new(
            service_account,
            Relation::Member,
            organization,
        )])
        .dispatch(&self.db)
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
        Operation::write_relationships(vec![Relationship::new(
            user,
            Relation::Member,
            organization,
        )])
        .dispatch(&self.db)
        .await?;

        Ok(())
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

                Organization {
                    id: Uuid::new_v4(),
                    name: organization_name,
                    slug,
                    parent_id: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
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
