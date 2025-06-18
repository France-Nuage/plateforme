use database::Persistable;
use derive_factory::Factory;
use derive_repository::Repository;
use resources::organizations::OrganizationFactory;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Repository)]
pub struct Hypervisor {
    /// The hypervisor id
    #[repository(primary)]
    pub id: Uuid,

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
