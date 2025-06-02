use database::{Factory, HasFactory};
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

use crate::organizations::{Organization, OrganizationFactory};

#[derive(Debug, Default, FromRow)]
pub struct Project {
    /// The project id
    pub id: Uuid,
    /// The project name
    pub name: String,
    /// The organization this project belongs to
    pub organization_id: Uuid,
    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// The HasFactory trait implementation for the project model.
impl HasFactory for Project {
    type Factory = ProjectFactory;

    /// Get a new factory instance for the model.
    fn factory(pool: PgPool) -> Self::Factory {
        ProjectFactory {
            pool,
            project: Project::default(),
            organization_factory: None,
        }
    }
}

/// The factory companion for the project model.
pub struct ProjectFactory {
    /// The database connection pool.
    pool: PgPool,

    /// The model to factorize.
    project: Project,

    /// The organization relation factory.
    organization_factory:
        Option<Box<dyn FnOnce(OrganizationFactory) -> OrganizationFactory + Send>>,
}

/// The Factory trait implementation for the project factory.
impl Factory for ProjectFactory {
    type Model = Project;

    /// Create a single project and persist it into the database.
    async fn create(mut self) -> Result<Self::Model, sqlx::Error> {
        // build the hypervisor relation if requested
        if let Some(configure) = self.organization_factory {
            let factory = Organization::factory(self.pool.clone());
            let factory = configure(factory);
            let model = factory.create().await?;
            self.project.organization_id = model.id;
        }

        crate::projects::repository::create(&self.pool, self.project).await
    }

    /// Add a new state transformation to the project definition.
    fn state(mut self, project: Project) -> Self {
        self.project = project;
        self
    }
}

impl ProjectFactory {
    /// Define a parent organization relationship for the model.
    pub fn for_organization(self) -> Self {
        self.for_organization_with(|factory| factory)
    }

    /// Define a parent organization relationship for the model.
    pub fn for_organization_with<F>(mut self, configure_organization: F) -> Self
    where
        F: FnOnce(OrganizationFactory) -> OrganizationFactory + Send + 'static,
    {
        self.organization_factory = Some(Box::new(configure_organization));
        self
    }
}
