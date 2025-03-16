use crate::error::Error;
use serde::{Deserialize, Serialize};

pub mod error;

/// Represents a hypervisor.
pub trait Cluster {
    /// Gets a node belonging to the hypervisor.
    fn node(&self, id: &str) -> impl Node;
}

/// Represents a node.
pub trait Node {
    /// Gets an instance belonging to the node.
    fn instance(&self, id: u32) -> impl Instance;

    /// Gets the instances belonging to the node.
    fn list_instances(&self) -> impl Future<Output = Result<(), Error>>;
}

pub trait Instance {
    /// Creates the instance.
    fn create(&self, options: &InstanceConfig) -> impl Future<Output = Result<(), Error>>;

    /// Deletes the instance.
    fn delete(&self) -> impl Future<Output = Result<(), Error>>;

    /// Starts the instance.
    fn start(&self) -> impl Future<Output = Result<(), Error>>;

    /// Gets the instance status.
    fn status(&self) -> impl Future<Output = Result<InstanceStatus, Error>>;

    /// Stops the instance.
    fn stop(&self) -> impl Future<Output = Result<(), Error>>;
}

pub struct InstanceConfig<'a> {
    /// The instance id.
    pub id: u32,

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
