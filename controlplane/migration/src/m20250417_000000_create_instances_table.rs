use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Instances::Table)
                    .if_not_exists()
                    .col(pk_uuid(Instances::Id))
                    .col(string(Instances::DistantId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Instances::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Instances {
    Table,
    Id,
    DistantId,
}
