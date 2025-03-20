use crate::error::Error;
use serde::{Deserialize, Serialize};

pub mod error;

/// Represents a hypervisor.
pub trait Cluster {
    /// Gets a node belonging to the hypervisor.
    fn node(&self, id: &str) -> impl Node + Send;

    fn instances(&self)
    -> impl Future<Output = Result<Vec<proto::v0::InstanceInfo>, Error>> + Send;
}

/// Represents a node.
pub trait Node {
    /// Gets an instance belonging to the node.
    fn instance(&self, id: &str) -> impl Instance + Send;

    /// Gets the instances belonging to the node.
    fn list_instances(&self) -> impl Future<Output = Result<(), Error>> + Send;
}

pub trait Instance {
    /// Creates the instance.
    fn create(&self, options: &InstanceConfig) -> impl Future<Output = Result<(), Error>> + Send;

    /// Deletes the instance.
    fn delete(&self) -> impl Future<Output = Result<(), Error>> + Send;

    /// Starts the instance.
    fn start(&self) -> impl Future<Output = Result<(), Error>> + Send;

    /// Gets the instance status.
    fn status(&self) -> impl Future<Output = Result<InstanceStatus, Error>> + Send;

    /// Stops the instance.
    fn stop(&self) -> impl Future<Output = Result<(), Error>> + Send;
}

pub struct InstanceConfig<'a> {
    /// The instance id.
    pub id: &'a str,

    /// The instance name.
    pub name: &'a str,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InstanceStatus {
    #[serde(rename = "running")]
    Running,

    #[serde(rename = "stopped")]
    Stopped,
}

impl From<InstanceStatus> for i32 {
    fn from(status: InstanceStatus) -> i32 {
        match status {
            InstanceStatus::Running => proto::v0::InstanceStatus::Running as i32,
            InstanceStatus::Stopped => proto::v0::InstanceStatus::Stopped as i32,
        }
    }
}
