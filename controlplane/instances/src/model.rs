//! Database model for instance entities.

use std::fmt::Display;

use database::Persistable;
use derive_factory::Factory;
use derive_repository::Repository;
use hypervisors::HypervisorFactory;
use resources::projects::ProjectFactory;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Default, Factory, sqlx::FromRow, Repository)]
pub struct Instance {
    /// Unique identifier for the instance
    #[repository(primary)]
    pub id: Uuid,
    /// The hypervisor this instance is attached to
    #[factory(relation = "HypervisorFactory")]
    pub hypervisor_id: Uuid,
    /// The project this instance belongs to
    #[factory(relation = "ProjectFactory")]
    pub project_id: Uuid,
    /// The zero trust network this instance belongs to
    pub zero_trust_network_id: Option<Uuid>,
    /// ID used by the hypervisor to identify this instance remotely
    pub distant_id: String,
    /// Current CPU utilization as a percentage (0.0-100.0)
    pub cpu_usage_percent: f64,
    /// Current disk utilization (in bytes, cannot exceed max_disk_bytes)
    pub disk_usage_bytes: i64,
    /// IP address v4
    pub ip_v4: String,
    /// Maximum CPU cores available to the instance (max 99)
    pub max_cpu_cores: i32,
    /// Maximum disk available to the instance (in bytes, max 100TB)
    pub max_disk_bytes: i64,
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

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
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
