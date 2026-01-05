use chrono::{DateTime, Utc};
use fabrique::{Factory, Model};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, PartialEq)]
pub struct ZeroTrustNetworkType {
    /// Unique identifier for the zero trust network
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// Zero trust network type name
    pub name: String,

    /// Creation time of the zero trust network type name
    pub created_at: DateTime<Utc>,

    /// Time of the zero trust network type name last update
    pub updated_at: DateTime<Utc>,
}
