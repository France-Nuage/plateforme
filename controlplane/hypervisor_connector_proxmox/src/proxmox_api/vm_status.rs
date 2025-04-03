use hypervisor_connector::InstanceStatus;
use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum VMStatus {
    #[serde(rename = "running")]
    Running,

    #[serde(rename = "stopped")]
    Stopped,
}

impl From<VMStatus> for InstanceStatus {
    fn from(value: VMStatus) -> Self {
        match value {
            VMStatus::Running => InstanceStatus::Running,
            VMStatus::Stopped => InstanceStatus::Stopped,
        }
    }
}
