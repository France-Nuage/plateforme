use crate::Error;
use crate::resourcemanager::OrganizationFactory;
use database::{Factory, Persistable, Repository};
use frn_core::authorization::Resource;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Repository, Resource)]
pub struct Project {
    /// The project id
    #[repository(primary)]
    pub id: Uuid,

    /// The project name
    pub name: String,

    /// The organization this project belongs to
    #[factory(relation = "OrganizationFactory")]
    pub organization_id: Uuid,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_project(
    executor: &Pool<Postgres>,
    name: String,
    organization_id: Uuid,
) -> Result<Project, Error> {
    Project::factory()
        .name(name)
        .organization_id(organization_id)
        .create(executor)
        .await
        .map_err(Into::into)
}

pub async fn list_projects(executor: &Pool<Postgres>) -> Result<Vec<Project>, Error> {
    Project::list(executor).await.map_err(Into::into)
}
