use crate::ZeroTrustNetworkTypeFactory;
use database::Persistable;
use derive_factory::Factory;
use derive_repository::Repository;
use resources::organizations::OrganizationFactory;
use sqlx::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, PartialEq, Repository)]
pub struct ZeroTrustNetwork {
    /// Unique identifier for the zero trust network
    #[repository(primary)]
    pub id: Uuid,

    #[factory(relation = "OrganizationFactory")]
    pub organization_id: Uuid,

    #[factory(relation = "ZeroTrustNetworkTypeFactory")]
    pub zero_trust_network_type_id: Uuid,

    /// Zero trust network name
    pub name: String,

    // Creation time of the zero trust network type
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Time of the zero trust network type last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
