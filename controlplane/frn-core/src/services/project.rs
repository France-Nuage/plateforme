use database::Persistable;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{Error, models::Project};

pub async fn create(
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

pub async fn list(executor: &Pool<Postgres>) -> Result<Vec<Project>, Error> {
    Project::list(executor).await.map_err(Into::into)
}
