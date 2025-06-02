//! Database model for instance entities.

use std::fmt::Display;

use database::{Factory, HasFactory};
use hypervisors::{Hypervisor, HypervisorFactory};
use resources::projects::{Project, ProjectFactory};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::chrono};
use uuid::Uuid;

#[derive(Debug, Default, sqlx::FromRow)]
pub struct Instance {
    /// Unique identifier for the instance
    pub id: Uuid,
    /// The hypervisor this instance is attached to
    pub hypervisor_id: Uuid,
    /// The project this instance belongs to
    pub project_id: Uuid,
    /// ID used by the hypervisor to identify this instance remotely
    pub distant_id: String,
    /// Current CPU utilization as a percentage (0.0-100.0)
    pub cpu_usage_percent: f64,
    /// Maximum CPU cores available to the instance (max 99)
    pub max_cpu_cores: i32,
    /// Maximum memory available to the instance (in bytes, max 64GB)
    pub max_memory_bytes: i64,
    /// Current memory utilization (in bytes, cannot exceed max_memory_bytes)
    pub memory_usage_bytes: i64,
    /// Human-readable name, defined on the instance
    pub name: String,
    /// Current operational status of the instance
    #[sqlx(try_from = "String")]
    pub status: InstanceStatus,
    // Creation time of the instance
    pub created_at: chrono::DateTime<chrono::Utc>,
    // Time of the instance last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InstanceStatus {
    #[default]
    Unknown,
    Running,
    Stopped,
}

impl From<hypervisor_connector::InstanceStatus> for InstanceStatus {
    fn from(value: hypervisor_connector::InstanceStatus) -> Self {
        match value {
            hypervisor_connector::InstanceStatus::Running => InstanceStatus::Running,
            hypervisor_connector::InstanceStatus::Stopped => InstanceStatus::Stopped,
            hypervisor_connector::InstanceStatus::Unknown => InstanceStatus::Unknown,
        }
    }
}

impl From<InstanceStatus> for String {
    fn from(value: InstanceStatus) -> Self {
        serde_plain::to_string(&value).expect("Could not serialize an InstanceStatus into a string")
    }
}

impl From<String> for InstanceStatus {
    fn from(value: String) -> Self {
        serde_plain::from_str(&value)
            .expect("could not deserialize a string into an InstanceStatus")
    }
}

impl Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_plain::to_string(self)
                .expect("could not serialize an InstanceStatus into a string")
                .as_ref(),
        )
    }
}

/// The HasFactory trait implementation for the instance model.
impl HasFactory for Instance {
    type Factory = InstanceFactory;

    /// Get a new factory instance for the model.
    fn factory(pool: PgPool) -> Self::Factory {
        InstanceFactory {
            pool,
            instance: Instance::default(),
            hypervisor_factory: None,
            project_factory: None,
        }
    }
}

/// The factory companion for the instance model.
pub struct InstanceFactory {
    /// The database connection pool.
    pool: PgPool,

    /// The model to factorize.
    instance: Instance,

    /// The hypervisor relation factory.
    hypervisor_factory: Option<Box<dyn FnOnce(HypervisorFactory) -> HypervisorFactory + Send>>,

    /// The project relation factory.
    project_factory: Option<Box<dyn FnOnce(ProjectFactory) -> ProjectFactory + Send>>,
}

/// The Factory trait implementation for the instance factory.
impl Factory for InstanceFactory {
    type Model = Instance;

    /// Create a single instance and persist it into the database.
    async fn create(mut self) -> Result<Self::Model, sqlx::Error> {
        // build the hypervisor relation if requested
        if let Some(factorize) = self.hypervisor_factory {
            let factory = Hypervisor::factory(self.pool.clone());
            let factory = factorize(factory);
            let model = factory.create().await?;
            self.instance.hypervisor_id = model.id;
        }

        // build the project relation if requested
        if let Some(factorize) = self.project_factory {
            let factory = Project::factory(self.pool.clone());
            let factory = factorize(factory);
            let model = factory.create().await?;
            self.instance.project_id = model.id;
        }

        crate::repository::create(&self.pool, self.instance).await
    }

    /// Add a new state transformation to the instance definition.
    fn state(mut self, instance: Instance) -> Self {
        self.instance = instance;
        self
    }
}

/// Custom methods for the instance factory companion.
impl InstanceFactory {
    // Define a parent hypervisor relationship for the model.
    pub fn for_hypervisor(self) -> Self {
        self.for_hypervisor_with(|factory| factory)
    }

    /// Define a parent hypervisor relationship for the model with a customized factory.
    pub fn for_hypervisor_with<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(HypervisorFactory) -> HypervisorFactory + Send + 'static,
    {
        self.hypervisor_factory = Some(Box::new(configure));
        self
    }

    // Define a parent project relationship for the model.
    pub fn for_project(self) -> Self {
        self.for_project_with(|factory| factory)
    }

    /// Define a parent project relationship for the model with a customized factory.
    pub fn for_project_with<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(ProjectFactory) -> ProjectFactory + Send + 'static,
    {
        self.project_factory = Some(Box::new(configure));
        self
    }
}
