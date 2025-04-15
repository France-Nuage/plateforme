use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Hypervisors::Table)
                    .if_not_exists()
                    .col(pk_uuid(Hypervisors::Id))
                    .col(string(Hypervisors::Url))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Hypervisors::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Hypervisors {
    Table,
    Id,
    Url,
}
