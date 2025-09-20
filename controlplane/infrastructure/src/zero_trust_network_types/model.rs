use database::{Factory, Persistable, Repository};
use sqlx::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, PartialEq, Repository)]
pub struct ZeroTrustNetworkType {
    /// Unique identifier for the zero trust network
    #[repository(primary)]
    pub id: Uuid,

    /// Zero trust network type name
    pub name: String,

    // Creation time of the zero trust network type name
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Time of the zero trust network type name last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
