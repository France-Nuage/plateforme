use crate::InstanceStatus;

pub struct InstanceInfo {
    /// Unique identifier for the instance
    pub id: String,

    /// Maximum disk available to the instance (in bytes, max 100TB)
    pub disk_usage_bytes: u64,

    /// Maximum CPU cores available to the instance (max 99)
    pub max_cpu_cores: u32,

    /// Current CPU utilization as a percentage (0.0-100.0)
    pub cpu_usage_percent: f32,

    /// Maximum disk available to the instance (in bytes, max 100TB)
    pub max_disk_bytes: u64,

    /// Maximum memory available to the instance (in bytes, max 64GB)
    pub max_memory_bytes: u64,

    /// Current memory utilization (in bytes, cannot exceed max_memory_bytes)
    pub memory_usage_bytes: u64,

    /// Human readable name
    pub name: String,

    /// Current operational status of the instance
    pub status: InstanceStatus,
}
