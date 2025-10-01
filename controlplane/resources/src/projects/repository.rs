use frn_core::resourcemanager::Project;
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
pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Project>, sqlx::Error> {
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
pub async fn create(pool: &sqlx::PgPool, project: Project) -> Result<Project, sqlx::Error> {
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
}

/// Retrieves a single project.
///
/// # Arguments
///
/// * `name` - The project name
///
/// # Returns
///
/// * `Ok(Project)` - The requested project
/// * `Err(sqlx::Error)` - If the retrieval fails`
pub async fn find_one_by<'a>(pool: &PgPool, query: Query<'a>) -> Result<Project, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"
        SELECT id, name, organization_id, created_at, updated_at 
        FROM projects 
        WHERE $1::text IS NULL OR name = $1
        "#,
        query.name
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, Default)]
pub struct Query<'a> {
    pub name: Option<&'a str>,
    pub organization_id: Option<&'a Uuid>,
}
