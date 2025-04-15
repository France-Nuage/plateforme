use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Default, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "hypervisors", schema_name = "public")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub url: String,
    pub authentication_token: String,
    pub storage_name: String,
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
