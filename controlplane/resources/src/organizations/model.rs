use database::Persistable;
use derive_factory::Factory;
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow)]
pub struct Organization {
    /// The organization id
    pub id: Uuid,
    /// The organization name
    pub name: String,
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Persistable for Organization {
    type Connection = sqlx::PgPool;
    type Error = sqlx::Error;

    /// Create a new organization record in the database.
    async fn create(self, pool: PgPool) -> Result<Self, Self::Error> {
        crate::organizations::repository::create(&pool, self).await
    }

    /// Update an existing organization record in the database.
    async fn update(self, _pool: PgPool) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
