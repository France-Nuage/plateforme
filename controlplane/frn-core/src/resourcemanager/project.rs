use crate::Error;
use crate::authorization::{Authorize, Principal};
use crate::resourcemanager::OrganizationFactory;
use database::{Factory, Persistable, Repository};
use frn_core::authorization::Resource;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub const DEFAULT_PROJECT_NAME: &str = "unattributed";

#[derive(Debug, Default, Factory, FromRow, Repository, Resource)]
pub struct Project {
    /// The project id
    #[repository(primary)]
    pub id: Uuid,

    /// The project name
    pub name: String,

    /// The organization this project belongs to
    #[factory(relation = "OrganizationFactory")]
    pub organization_id: Uuid,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ProjectCreateRequest {
    name: String,
    organization_id: Uuid,
}

#[derive(Clone)]
pub struct Projects<Auth: Authorize> {
    _auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Projects<Auth> {
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { _auth: auth, db }
    }

    pub async fn create<P: Principal>(
        &mut self,
        _principal: &P,
        request: ProjectCreateRequest,
    ) -> Result<Project, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&Project::any())
        //     .await?;

        Project::factory()
            .id(Uuid::new_v4())
            .name(request.name)
            .organization_id(request.organization_id)
            .create(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_default_project<P: Principal>(
        &mut self,
        _principal: &P,
        organization_id: &Uuid,
    ) -> Result<Project, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Get)
        //     .over(&Project::any())
        //     .await?;

        sqlx::query_as!(
            Project,
            "SELECT * FROM projects WHERE organization_id = $1 AND name = $2",
            organization_id,
            DEFAULT_PROJECT_NAME,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Into::into)
    }
}

pub async fn create_project(
    executor: &Pool<Postgres>,
    name: String,
    organization_id: Uuid,
) -> Result<Project, Error> {
    Project::factory()
        .name(name)
        .organization_id(organization_id)
        .create(executor)
        .await
        .map_err(Into::into)
}

pub async fn list_projects(executor: &Pool<Postgres>) -> Result<Vec<Project>, Error> {
    Project::list(executor).await.map_err(Into::into)
}
