//! Tests for the bootstrap-admin seeding done at control-plane startup.
//!
//! A fresh install must end up with exactly one founding platform admin, and
//! re-running the bootstrap (every boot) must be a no-op. Drives
//! `Users::initialize_root_admin` against a real database.

use frn_core::identity::{User, Users};
use spicedb::SpiceDB;

/// On a fresh database the configured email is created as a platform admin.
#[sqlx::test(migrations = "../migrations")]
async fn test_bootstraps_a_new_admin(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let users = Users::new(SpiceDB::mock().await, pool.clone());

    let admin = users
        .initialize_root_admin("founder@france-nuage.fr".to_owned())
        .await?;

    assert!(admin.is_admin, "the bootstrapped user must be an admin");
    assert_eq!(admin.email, "founder@france-nuage.fr");
    // The subject must stay unpinned (NULL) until the admin's first real login:
    // a Faker-generated subject would never match the IdP token and fail closed
    // with SubjectMismatch, locking the admin out.
    assert_eq!(
        admin.sub, None,
        "the bootstrapped admin must have no pinned subject until first login"
    );

    let persisted = User::find_one_by_email(&pool, "founder@france-nuage.fr")
        .await?
        .expect("the admin must be persisted");
    assert!(persisted.is_admin);
    assert_eq!(
        persisted.sub, None,
        "the persisted admin subject must be NULL"
    );
    Ok(())
}

/// An existing non-admin user (e.g. seeded by a prior login) is promoted, in
/// place, without creating a duplicate row.
#[sqlx::test(migrations = "../migrations")]
async fn test_promotes_an_existing_non_admin(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let users = Users::new(SpiceDB::mock().await, pool.clone());

    // First login created a regular user (pinned to its IdP subject, as the
    // real login path does).
    let existing =
        User::find_or_create_one_by_email(&pool, "founder@france-nuage.fr", "idp-subject-founder")
            .await?;
    assert!(!existing.is_admin, "precondition: user starts non-admin");

    let admin = users
        .initialize_root_admin("founder@france-nuage.fr".to_owned())
        .await?;

    assert_eq!(admin.id, existing.id, "the same row must be promoted");
    assert!(admin.is_admin);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1::citext")
        .bind("founder@france-nuage.fr")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 1, "promotion must not create a duplicate user");
    Ok(())
}

/// Running the bootstrap again on an already-admin user is a no-op.
#[sqlx::test(migrations = "../migrations")]
async fn test_is_idempotent_for_an_existing_admin(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let users = Users::new(SpiceDB::mock().await, pool.clone());

    let first = users
        .initialize_root_admin("founder@france-nuage.fr".to_owned())
        .await?;
    let second = users
        .initialize_root_admin("founder@france-nuage.fr".to_owned())
        .await?;

    assert_eq!(first.id, second.id, "no new row on the second run");
    assert!(second.is_admin);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 1, "exactly one user must exist");
    Ok(())
}
