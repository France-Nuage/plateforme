use sqlx::PgPool;

use crate::{
    Problem,
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
                name: Some("unattributed"),
            },
        )
        .await
        .map_err(Into::into)
    }

    pub fn new(pool: PgPool) -> ResourcesService {
        ResourcesService { pool }
    }
}
