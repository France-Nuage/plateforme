use crate::{ZeroTrustNetworkType, ZeroTrustNetworkTypeFactory, ZeroTrustNetworkTypeIdColumn};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Model};
use frn_core::resourcemanager::{Organization, OrganizationFactory, OrganizationIdColumn};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, PartialEq)]
pub struct ZeroTrustNetwork {
    /// Unique identifier for the zero trust network
    #[fabrique(primary_key)]
    pub id: Uuid,

    #[fabrique(belongs_to = Organization)]
    pub organization_id: Uuid,

    #[fabrique(belongs_to = ZeroTrustNetworkType)]
    pub zero_trust_network_type_id: Uuid,

    /// Zero trust network name
    pub name: String,

    /// Creation time of the zero trust network type
    pub created_at: DateTime<Utc>,

    /// Time of the zero trust network type last update
    pub updated_at: DateTime<Utc>,
}
