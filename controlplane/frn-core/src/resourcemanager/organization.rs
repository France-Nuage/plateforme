use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::identity::{ServiceAccount, User};
use database::{Factory, Persistable, Repository};
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Repository, Resource)]
pub struct Organization {
    /// The organization id
    #[repository(primary)]
    pub id: Uuid,
    /// The organization name
    pub name: String,
    /// The organization parent, if any
    pub parent_id: Option<Uuid>,
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

        Organization::factory()
            .id(Uuid::new_v4())
            .name(name)
            .parent_id(parent_id)
            .create(connection)
            .await
            .map_err(Into::into)
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

    pub async fn initialize_root_organization(
        &mut self,
        organization_name: String,
    ) -> Result<Organization, Error> {
        // Attempt to retrieve the organization from the database
        let maybe_organization = sqlx::query_as!(
            Organization,
            "SELECT * FROM organizations WHERE name = $1 LIMIT 1",
            organization_name
        )
        .fetch_optional(&self.db)
        .await?;

        // Create the root organization if there is no database match
        let organization = match maybe_organization {
            Some(organization) => organization,
            None => {
                Organization::factory()
                    .name(organization_name)
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
