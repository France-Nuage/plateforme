use database::{Factory, Persistable, Repository};
use frn_core::models::OrganizationFactory;
use infrastructure::DatacenterFactory;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Repository)]
pub struct Hypervisor {
    /// The hypervisor id
    #[repository(primary)]
    pub id: Uuid,

    #[factory(relation = "DatacenterFactory")]
    pub datacenter_id: Uuid,

    /// The id of the organization the hypervisor belongs to
    #[factory(relation = "OrganizationFactory")]
    pub organization_id: Uuid,

    /// The hypervisor url
    pub url: String,

    /// The hypervisor authentication token
    pub authorization_token: String,

    /// The hypervisor storage name
    pub storage_name: String,
}
