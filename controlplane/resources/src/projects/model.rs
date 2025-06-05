use database::Persistable;
use derive_factory::Factory;
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

use crate::organizations::OrganizationFactory;

#[derive(Debug, Default, Factory, FromRow)]
pub struct Project {
    /// The project id
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

impl Persistable for Project {
    type Connection = sqlx::PgPool;
    type Error = sqlx::Error;

    /// Create a new project record in the database.
    async fn create(self, pool: PgPool) -> Result<Self, Self::Error> {
        crate::projects::repository::create(&pool, self).await
    }

    /// Update an existing project record in the database.
    async fn update(self, _pool: PgPool) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
