use frn_core::resourcemanager::Organization;
use sqlx::PgPool;
use uuid::Uuid;

/// Retrieves all organizations from the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
///
/// A vector of all organization records or a Problem if the operation fails
pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Organization>, sqlx::Error> {
    sqlx::query_as!(
        Organization,
        r#"
        SELECT id, name, created_at, updated_at
        FROM organizations
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Creates a new organization record in the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
/// * `organization` - The organization to be created
///
/// # Returns
///
/// The created organization on success or a Problem if the operation fails
pub async fn create(
    pool: &sqlx::PgPool,
    organization: Organization,
) -> Result<Organization, sqlx::Error> {
    sqlx::query_as!(
        Organization,
        r#"
        INSERT INTO organizations (id, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, created_at, updated_at
        "#,
        organization.id,
        organization.name,
        organization.created_at,
        organization.updated_at
    )
    .fetch_one(pool)
    .await
}

/// Retrieves a single organization by ID.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `id` - UUID of the organization to retrieve
///
/// # Returns
///
/// * `Ok(Organization)` - The requested organization
/// * `Err(Problem)` - If retrieval fails or organization doesn't exist
pub async fn get(pool: &PgPool, id: Uuid) -> Result<Organization, sqlx::Error> {
    sqlx::query_as!(
        Organization,
        r#"
        SELECT id, name, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}
