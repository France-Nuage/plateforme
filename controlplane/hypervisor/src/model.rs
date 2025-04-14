use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "hypervisors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
