use std::net::Ipv4Addr;

use crate::InstanceInfo;
use crate::instance_config::InstanceConfig;
use crate::instance_status::InstanceStatus;
use crate::problem::Problem;

pub trait InstanceService {
    /// Lists all instances.
    fn list(&self) -> impl Future<Output = Result<Vec<InstanceInfo>, Problem>> + Send;

    /// Clone the instance.
    fn clone(&self, source_id: &str) -> impl Future<Output = Result<String, Problem>> + Send;

    /// Creates the instance.
    fn create(
        &self,
        options: InstanceConfig,
    ) -> impl Future<Output = Result<String, Problem>> + Send;

    /// Gets the instance ip address.
    fn get_ip_address(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<Option<Ipv4Addr>, Problem>> + Send;

    /// Deletes the instance.
    fn delete(&self, id: &str) -> impl Future<Output = Result<(), Problem>> + Send;

    /// Starts the instance.
    fn start(&self, id: &str) -> impl Future<Output = Result<(), Problem>> + Send;

    /// Gets the instance status.
    fn status(&self, id: &str) -> impl Future<Output = Result<InstanceStatus, Problem>> + Send;

    /// Stops the instance.
    fn stop(&self, id: &str) -> impl Future<Output = Result<(), Problem>> + Send;
}
