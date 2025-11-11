use crate::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Status {
    /// Instance is active and operational.
    Running,

    /// Instance is inactive.
    Stopped,

    /// Instance status is unknown.
    #[default]
    Unknown,
}

impl From<String> for Status {
    fn from(value: String) -> Self {
        Status::from_str(&value).expect("could not parse value to status")
    }
}

impl From<Status> for String {
    fn from(value: Status) -> Self {
        value.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
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
    pub status: Status,
}

pub struct InstanceCreateRequest {
    /// The instance unique id.
    pub id: String,

    /// The number of cores per socket.
    pub cores: u8,

    /// The disk size.
    pub disk_bytes: u32,

    /// The disk image to create the instance from.
    pub disk_image: String,

    /// Memory properties.
    pub memory_bytes: u32,

    /// The instance human-readable name.
    pub name: String,

    /// The Cloud-Init snippet.
    pub snippet: String,
}

pub trait Instances: Clone {
    /// Lists all instances.
    fn list(&self) -> impl Future<Output = Result<Vec<Instance>, Error>> + Send;

    /// Clone the instance.
    fn clone(&self, source_id: &str) -> impl Future<Output = Result<String, Error>> + Send;

    /// Creates the instance.
    fn create(
        &self,
        options: InstanceCreateRequest,
    ) -> impl Future<Output = Result<Uuid, Error>> + Send;

    /// Gets the instance ip address.
    fn get_ip_address(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<Option<Ipv4Addr>, Error>> + Send;

    /// Deletes the instance.
    fn delete(&self, id: &str) -> impl Future<Output = Result<(), Error>> + Send;

    /// Starts the instance.
    fn start(&self, id: &str) -> impl Future<Output = Result<(), Error>> + Send;

    /// Gets the instance status.
    fn status(&self, id: &str) -> impl Future<Output = Result<Status, Error>> + Send;

    /// Stops the instance.
    fn stop(&self, id: &str) -> impl Future<Output = Result<(), Error>> + Send;
}
