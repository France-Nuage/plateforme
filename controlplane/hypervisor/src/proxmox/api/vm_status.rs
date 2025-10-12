use crate::instance::Status;
use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum VMStatus {
    #[serde(rename = "running")]
    Running,

    #[serde(rename = "stopped")]
    Stopped,

    #[serde(rename = "unknown")]
    Unknown,
}

impl From<VMStatus> for Status {
    fn from(value: VMStatus) -> Self {
        match value {
            VMStatus::Running => Status::Running,
            VMStatus::Stopped => Status::Stopped,
            VMStatus::Unknown => Status::Unknown,
        }
    }
}
