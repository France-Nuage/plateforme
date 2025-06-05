use database::Persistable;
use derive_factory::Factory;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow)]
pub struct Hypervisor {
    pub id: Uuid,
    pub url: String,
    pub authorization_token: String,
    pub storage_name: String,
}

impl Persistable for Hypervisor {
    type Connection = sqlx::PgPool;
    type Error = sqlx::Error;

    /// Create a new hypervisor record in the database.
    async fn create(self, pool: PgPool) -> Result<Self, Self::Error> {
        crate::repository::create(&pool, self).await
    }

    /// Update an existing hypervisor record in the database.
    async fn update(self, _pool: PgPool) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
