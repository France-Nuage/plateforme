use database::{Factory, Persistable, Repository};
use sqlx::{prelude::FromRow, types::chrono};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, PartialEq, Repository)]
pub struct Datacenter {
    /// Unique identifier for the datacenter
    #[repository(primary)]
    pub id: Uuid,

    /// A human-readable name for the datacenter
    pub name: String,

    // Creation time of the datacenter
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Time of the datancenter last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
