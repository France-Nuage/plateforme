tonic::include_proto!("francenuage.fr.api.controlplane.v1.instances");

/// Converts a InstanceInfo struct into a protocol compatible `v1::InstanceInfo`.
impl From<hypervisor_connector::InstanceInfo> for InstanceInfo {
    fn from(value: hypervisor_connector::InstanceInfo) -> Self {
        InstanceInfo {
            id: value.id,
            status: value.status as i32,
            max_cpu_cores: value.max_cpu_cores,
            cpu_usage_percent: value.cpu_usage_percent,
            max_memory_bytes: value.max_memory_bytes,
            memory_usage_bytes: value.memory_usage_bytes,
        }
    }
}

/// Converts a `hypervisor_connector::InstanceStatus` into a protocol compatible
/// `v1::InstanceStatus`.
impl From<hypervisor_connector::InstanceStatus> for InstanceStatus {
    fn from(value: hypervisor_connector::InstanceStatus) -> Self {
        match value {
            hypervisor_connector::InstanceStatus::Running => InstanceStatus::Running,
            hypervisor_connector::InstanceStatus::Stopped => InstanceStatus::Stopped,
        }
    }
}
