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
