use frn_core::models::Project;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{DEFAULT_PROJECT_NAME, Problem, projects::repository::Query};

pub struct ResourcesService {
    pool: PgPool,
}

impl ResourcesService {
    pub async fn get_default_project(&self, organization_id: &Uuid) -> Result<Project, Problem> {
        match crate::projects::repository::find_one_by(
            &self.pool,
            Query {
                name: Some(DEFAULT_PROJECT_NAME),
                organization_id: Some(organization_id),
            },
        )
        .await
        {
            Ok(project) => Ok(project),
            Err(sqlx::Error::RowNotFound) => self.create_default_project(*organization_id).await,
            Err(err) => Err(err.into()),
        }
    }

    pub async fn create_default_project(&self, organization_id: Uuid) -> Result<Project, Problem> {
        crate::projects::repository::create(
            &self.pool,
            Project {
                id: Uuid::new_v4(),
                name: DEFAULT_PROJECT_NAME.to_owned(),
                organization_id,
                ..Default::default()
            },
        )
        .await
        .map_err(Into::into)
    }

    pub fn new(pool: PgPool) -> ResourcesService {
        ResourcesService { pool }
    }
}
