use frn_core::resourcemanager::Project;
use sqlx::PgPool;

use crate::Problem;

pub struct ProjectService {
    pool: PgPool,
}

impl ProjectService {
    /// List all projects.
    pub async fn list(&self) -> Result<Vec<Project>, Problem> {
        crate::projects::repository::list(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// Create a new project.
    pub async fn create(&self, project: Project) -> Result<Project, Problem> {
        crate::projects::repository::create(&self.pool, project)
            .await
            .map_err(Into::into)
    }

    /// Create a new project service.
    pub fn new(pool: PgPool) -> ProjectService {
        ProjectService { pool }
    }
}
