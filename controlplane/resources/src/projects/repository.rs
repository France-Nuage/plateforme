use crate::projects::{model::Project, problem::Problem};
use sqlx::PgPool;
use uuid::Uuid;

/// Retrieves all projects from the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
///
/// A vector of all project records or a problem if the operation fails
pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Project>, Problem> {
    sqlx::query_as!(
        Project,
        r#"
        SELECT id, name, organization_id, created_at, updated_at
        FROM projects
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|err| Problem::DatabaseError(err.to_string()))
}

/// Lists projects filtered by organization ID.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
/// * `organization_id` - UUID of the organization to filter by
///
/// # Returns
///
/// A vector of filtered project records or a problem if the operation fails
pub async fn list_by_organization(
    pool: &sqlx::PgPool,
    organization_id: Uuid,
) -> Result<Vec<Project>, Problem> {
    sqlx::query_as!(
        Project,
        r#"
        SELECT id, name, organization_id, created_at, updated_at
        FROM projects
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
        organization_id
    )
    .fetch_all(pool)
    .await
    .map_err(|err| Problem::DatabaseError(err.to_string()))
}

/// Creates a new project record in the database.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
/// * `project` - The project to be created
///
/// # Returns
///
/// The created project on success or a problem if the operation fails
pub async fn create(pool: &sqlx::PgPool, project: &Project) -> Result<Project, Problem> {
    sqlx::query_as!(
        Project,
        r#"
        INSERT INTO projects (id, name, organization_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, organization_id, created_at, updated_at
        "#,
        project.id,
        project.name,
        project.organization_id,
        project.created_at,
        project.updated_at
    )
    .fetch_one(pool)
    .await
    .map_err(|err| Problem::DatabaseError(err.to_string()))
}

/// Retrieves a single project by ID.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `id` - UUID of the project to retrieve
///
/// # Returns
///
/// * `Ok(Project)` - The requested project
/// * `Err(Problem)` - If retrieval fails or project doesn't exist
pub async fn get(pool: &PgPool, id: Uuid) -> Result<Project, Problem> {
    sqlx::query_as!(
        Project,
        r#"
        SELECT id, name, organization_id, created_at, updated_at
        FROM projects
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => Problem::ProjectNotFound(id.to_string()),
        _ => Problem::DatabaseError(err.to_string()),
    })
}
