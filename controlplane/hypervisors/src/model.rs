use database::{Factory, HasFactory};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Default, FromRow)]
pub struct Hypervisor {
    pub id: Uuid,
    pub url: String,
    pub authorization_token: String,
    pub storage_name: String,
}

/// The HasFactory trait implementation for the instance model.
impl HasFactory for Hypervisor {
    type Factory = HypervisorFactory;

    /// Get a new factory instance for the model.
    fn factory(pool: PgPool) -> Self::Factory {
        HypervisorFactory {
            pool,
            hypervisor: Hypervisor::default(),
        }
    }
}

/// The factory companion for the instance model.
pub struct HypervisorFactory {
    /// The database connection pool.
    pool: PgPool,

    /// The model to factorize.
    hypervisor: Hypervisor,
}

/// The Factory trait implementation for the instance factory.
impl Factory for HypervisorFactory {
    type Model = Hypervisor;

    /// Create a single instance and persist it into the database.
    async fn create(self) -> Result<Self::Model, sqlx::Error> {
        crate::repository::create(&self.pool, self.hypervisor).await
    }

    /// Add a new state transformation to the instance definition.
    fn state(mut self, hypervisor: Hypervisor) -> Self {
        self.hypervisor = hypervisor;
        self
    }
}
