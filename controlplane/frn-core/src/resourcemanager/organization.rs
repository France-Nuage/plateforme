use crate::Error;
use crate::authorization::{Authorize, Principal, Resource};
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
pub struct Organizations<A: Authorize> {
    _auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Organizations<A> {
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { _auth: auth, db }
    }

    pub async fn list_organizations<P: Principal>(
        &mut self,
        principal: &P,
    ) -> Result<Vec<Organization>, Error> {
        // TODO: Implement lookup functionality in new authorization API
        // let verbs = self
        //     .auth
        //     .check::<P, Organization>(principal.id())
        //     .perform(Permission::Get)
        //     .over(&Organization::some(""))
        //     .lookup::<Organization>(&self.db)
        //     .await?;
        //
        // tracing::info!("data: {:?}", verbs);

        principal.organizations(&self.db).await
    }

    pub async fn create_organization<P: Principal + Sync>(
        &mut self,
        connection: &Pool<Postgres>,
        _principal: &P,
        name: String,
    ) -> Result<Organization, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&Organization::any())
        //     .await?;

        Organization::factory()
            .name(name)
            .create(connection)
            .await
            .map_err(Into::into)
    }
}
