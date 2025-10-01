use crate::Error;
use crate::iam::Principal;
use database::{Factory, Persistable, Repository};
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Repository)]
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

pub async fn list_organizations<P: Principal>(
    connection: &Pool<Postgres>,
    principal: &P,
) -> Result<Vec<Organization>, Error> {
    principal.organizations(connection).await
}

pub async fn create_organization(
    connection: &Pool<Postgres>,
    name: String,
) -> Result<Organization, Error> {
    Organization::factory()
        .name(name)
        .create(connection)
        .await
        .map_err(Into::into)
}
