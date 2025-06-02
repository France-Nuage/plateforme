use database::{Factory, HasFactory};
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Default, FromRow)]
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

/// The HasFactory trait implementation for the organization model.
impl HasFactory for Organization {
    type Factory = OrganizationFactory;

    /// Get a new factory instance for the model.
    fn factory(pool: sqlx::PgPool) -> Self::Factory {
        OrganizationFactory {
            pool,
            organization: Organization::default(),
        }
    }
}

/// The factory companion for the organization model.
pub struct OrganizationFactory {
    /// The database connection pool.
    pool: PgPool,

    /// The model to factorize.
    organization: Organization,
}

/// The Factory trait implementation for the organization factory.
impl Factory for OrganizationFactory {
    type Model = Organization;

    /// Create a single project and persist it into the database.
    async fn create(self) -> Result<Self::Model, sqlx::Error> {
        crate::organizations::repository::create(&self.pool, self.organization).await
    }

    /// Add a new state transformation to the project definition.
    fn state(mut self, organization: Organization) -> Self {
        self.organization = organization;
        self
    }
}
