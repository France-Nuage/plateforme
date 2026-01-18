use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship};
use crate::resourcemanager::{Organization, OrganizationFactory, OrganizationIdColumn};
use fabrique::{Factory, Model};
use frn_core::authorization::Resource;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub const DEFAULT_PROJECT_NAME: &str = "unattributed";

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct Project {
    /// The project id
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The project name
    pub name: String,

    /// The organization this project belongs to
    #[fabrique(belongs_to = "Organization")]
    pub organization_id: Uuid,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ProjectCreateRequest {
    pub name: String,
    pub organization_id: Uuid,
}

#[derive(Clone)]
pub struct Projects<Auth: Authorize> {
    auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Projects<Auth> {
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    pub async fn list<P: Principal>(&mut self, principal: &P) -> Result<Vec<Project>, Error> {
        self.auth
            .lookup::<Project>()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await
    }

    pub async fn create<P: Principal>(
        &mut self,
        _principal: &P,
        request: ProjectCreateRequest,
    ) -> Result<Project, Error> {
        let project = Project::factory()
            .id(Uuid::new_v4())
            .name(request.name)
            .organization_id(request.organization_id)
            .create(&self.db)
            .await?;

        self.auth
            .write_relationship(&Relationship::new(
                &Organization::some(request.organization_id),
                Relation::Parent,
                &project,
            ))
            .await?;

        Ok(project)
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
