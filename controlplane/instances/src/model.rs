//! Database model for instance entities.

use std::fmt::Display;

use hypervisor_connector::InstanceInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, sqlx::FromRow)]
pub struct Instance {
    /// Unique identifier for the instance
    pub id: Uuid,
    /// Reference to the hypervisor hosting this instance
    pub hypervisor_id: Uuid,
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

impl From<InstanceInfo> for Instance {
    fn from(value: InstanceInfo) -> Self {
        Instance {
            id: Uuid::new_v4(),
            hypervisor_id: Uuid::default(),
            distant_id: value.id,
            cpu_usage_percent: value.cpu_usage_percent as f64,
            max_cpu_cores: value.max_cpu_cores as i32,
            max_memory_bytes: value.max_memory_bytes as i64,
            memory_usage_bytes: value.memory_usage_bytes as i64,
            name: value.name,
            status: value.status.into(),
        }
    }
}
