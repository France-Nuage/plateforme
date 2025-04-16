//! Database model for hypervisor entities.

use sea_orm::entity::prelude::*;

/// Represents a hypervisor entity in the database.
///
/// This model stores information about registered hypervisor platforms,
/// including their connection details and authentication information.
#[derive(Clone, Debug, Default, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "hypervisors", schema_name = "public")]
pub struct Model {
    /// Unique identifier for the hypervisor.
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// URL endpoint of the hypervisor API.
    pub url: String,

    /// Authentication token for secure API access.
    pub authentication_token: String,

    /// Name of the storage to use for instances on this hypervisor.
    pub storage_name: String,
}

/// Defines relations between the hypervisor entity and other entities.
///
/// Currently, the hypervisor entity has no relations to other entities.
#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

/// Implements behavior for active models of the hypervisor entity.
///
/// Uses the default behaviors provided by Sea ORM.
impl ActiveModelBehavior for ActiveModel {}
