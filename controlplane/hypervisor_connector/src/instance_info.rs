use crate::InstanceStatus;

pub struct InstanceInfo {
    /// Unique identifier for the instance
    pub id: String,

    /// Current operational status of the instance
    pub status: InstanceStatus,

    /// Maximum CPU cores available to the instance (max 99)
    pub max_cpu_cores: u32,

    /// Current CPU utilization as a percentage (0.0-100.0)
    pub cpu_usage_percent: f32,

    /// Maximum memory available to the instance (in bytes, max 64GB)
    pub max_memory_bytes: u64,

    /// Current memory utilization (in bytes, cannot exceed max_memory_bytes)
    pub memory_usage_bytes: u64,
}
