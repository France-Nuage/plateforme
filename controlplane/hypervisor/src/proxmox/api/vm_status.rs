use crate::instance::Status;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum ResourceStatus {
    #[serde(rename = "offline")]
    Offline,

    #[serde(rename = "online")]
    Online,

    #[serde(rename = "running")]
    Running,

    #[serde(rename = "stopped")]
    Stopped,

    #[serde(rename = "unknown")]
    Unknown,
}

impl From<ResourceStatus> for Status {
    fn from(value: ResourceStatus) -> Self {
        match value {
            ResourceStatus::Running => Status::Running,
            ResourceStatus::Stopped => Status::Stopped,
            ResourceStatus::Unknown => Status::Unknown,
            other => panic!("attempting to cast {:?} as an instance status", other),
        }
    }
}
