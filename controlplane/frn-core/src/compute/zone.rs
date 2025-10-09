use chrono::{DateTime, Utc};
use database::{Factory, Persistable, Repository};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, PartialEq, Repository)]
pub struct Zone {
    /// Unique identifier for the datacenter
    #[repository(primary)]
    pub id: Uuid,

    /// A human-readable name for the datacenter
    pub name: String,

    // Creation time of the datacenter
    pub created_at: DateTime<Utc>,

    // Time of the datancenter last update
    pub updated_at: DateTime<Utc>,
}
