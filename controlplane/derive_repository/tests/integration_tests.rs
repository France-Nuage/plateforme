use database_core::Persistable;
use derive_repository::Repository;
use sqlx::prelude::FromRow;

#[derive(Debug, Default, FromRow, Repository)]
struct Organization {
    #[repository(primary)]
    pub slug: String,

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
    let missile = Organization {
        slug: "test-org".to_owned(),
        ..Default::default()
    };

    let result = missile.create(&pool).await;

    assert!(result.is_ok());
}
