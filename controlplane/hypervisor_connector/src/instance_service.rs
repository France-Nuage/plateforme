use crate::InstanceInfo;
use crate::instance_config::InstanceConfig;
use crate::instance_status::InstanceStatus;
use crate::problem::Problem;

pub trait InstanceService {
    /// Lists all instances.
    fn list(&self) -> impl Future<Output = Result<Vec<InstanceInfo>, Problem>> + Send;

    /// Creates the instance.
    fn create(
        &self,
        options: InstanceConfig,
    ) -> impl Future<Output = Result<String, Problem>> + Send;

    /// Deletes the instance.
    fn delete(&self) -> impl Future<Output = Result<(), Problem>> + Send;

    /// Starts the instance.
    fn start(&self) -> impl Future<Output = Result<(), Problem>> + Send;

    /// Gets the instance status.
    fn status(&self) -> impl Future<Output = Result<InstanceStatus, Problem>> + Send;

    /// Stops the instance.
    fn stop(&self) -> impl Future<Output = Result<(), Problem>> + Send;
}
