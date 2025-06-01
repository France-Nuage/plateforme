use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Default, FromRow)]
pub struct Project {
    /// The project id
    pub id: Uuid,
    /// The project name
    pub name: String,
    /// The organization this project belongs to
    pub organization_id: Uuid,
    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
