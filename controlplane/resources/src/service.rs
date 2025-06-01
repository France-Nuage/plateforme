use sqlx::PgPool;

use crate::{
    DEFAULT_PROJECT_NAME, Problem,
    projects::{Project, repository::Query},
};

pub struct ResourcesService {
    pool: PgPool,
}

impl ResourcesService {
    pub async fn get_default_project(&self) -> Result<Project, Problem> {
        crate::projects::repository::find_one_by(
            &self.pool,
            Query {
                name: Some(DEFAULT_PROJECT_NAME),
            },
        )
        .await
        .map_err(Into::into)
    }

    pub fn new(pool: PgPool) -> ResourcesService {
        ResourcesService { pool }
    }
}
