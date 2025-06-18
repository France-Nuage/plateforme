use database::Persistable;
use derive_repository::Repository;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, FromRow, Repository)]
struct Organization {
    #[repository(primary)]
    pub id: Uuid,

    #[sqlx(try_from = "String")]
    pub name: OrganizationName,
}

#[derive(Debug, Default)]
enum OrganizationName {
    #[default]
    FranceNuage,
}

impl From<String> for OrganizationName {
    fn from(_: String) -> Self {
        OrganizationName::FranceNuage
    }
}

impl From<OrganizationName> for String {
    fn from(_: OrganizationName) -> Self {
        String::from("FranceNuage")
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn test_a_repository_can_be_derived_from_a_struct(pool: sqlx::PgPool) {
    // Arrange a missile
    let missile = Organization::default();

    // Act the call to the create method
    let result = missile.create(&pool).await;

    // Assert the result
    assert!(result.is_ok());
}
