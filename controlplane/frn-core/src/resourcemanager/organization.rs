use crate::Error;
use crate::authorization::{AuthorizationServer, Permission, Principal, Resource};
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
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct OrganizationService<A: AuthorizationServer> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: AuthorizationServer> OrganizationService<A> {
    pub async fn list_organizations<P: Principal + Sync>(
        &mut self,
        principal: &P,
    ) -> Result<Vec<Organization>, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Get)
            .over(&Organization::any())
            .await?;

        principal.organizations(&self.db).await
    }

    pub async fn create_organization<P: Principal + Sync>(
        &mut self,
        connection: &Pool<Postgres>,
        principal: &P,
        name: String,
    ) -> Result<Organization, Error> {
        self.auth
            .can(principal)
            .perform(Permission::Create)
            .over(&Organization::any())
            .await?;

        Organization::factory()
            .name(name)
            .create(connection)
            .await
            .map_err(Into::into)
    }
}
