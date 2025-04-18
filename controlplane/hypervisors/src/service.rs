use uuid::Uuid;

use crate::{model::Hypervisor, problem::Problem, repository};

pub struct HypervisorsService {
    pool: sqlx::PgPool,
}

impl HypervisorsService {
    pub async fn list(&self) -> Result<Vec<Hypervisor>, Problem> {
        repository::list(&self.pool).await.map_err(Problem::from)
    }

    pub async fn read(&self, id: Uuid) -> Result<Hypervisor, Problem> {
        repository::read(&self.pool, id)
            .await
            .map_err(Problem::from)
    }

    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}
