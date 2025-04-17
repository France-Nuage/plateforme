//! Database model for instance entities.

use sea_orm::entity::prelude::*;

/// Represents an instance entity in the database.
///
/// This model stores information about registered instances.
#[derive(Clone, Debug, Default, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "instances", schema_name = "public")]
pub struct Model {
    /// Unique identifier for the instance.
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// The instance id on the distant hypervisor.
    pub vendor_id: String,
}

/// Defines relations between the instance entity and other entities.
///
/// Currently, the hypervisor entity has no relations to other entities.
#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

/// Implements behavior for active models of the instance entity.
///
/// Uses the default behaviors provided by Sea ORM.
impl ActiveModelBehavior for ActiveModel {}
