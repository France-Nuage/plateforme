use std::time::Duration;

use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};

use super::{
    Problem,
    cluster_resources_list::ResourceType,
    task_status_read::{TaskStatus, TaskStatusResponse},
};

pub async fn wait_for_task_completion(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node: &str,
    task: &str,
) -> Result<TaskStatusResponse, Problem> {
    let strategy = ExponentialBackoff::from_millis(1000)
        .factor(2)
        .max_delay(Duration::from_secs(60))
        .map(jitter)
        .take(10);

    Retry::spawn(strategy, || async {
        let result =
            crate::proxmox_api::task_status_read(api_url, client, authorization, node, task).await;

        match result {
            Ok(response) => match response.data.status {
                TaskStatus::Running => Err(Problem::TaskNotCompleted(task.to_owned())),
                TaskStatus::Stopped => Ok(response.data),
            },
            Err(err) => Err(err),
        }
    })
    .await
}

pub async fn get_vm_execution_node(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    vmid: u32,
) -> Result<String, Problem> {
    let resource = crate::proxmox_api::cluster_resources_list(api_url, client, authorization, "vm")
        .await?
        .data
        .into_iter()
        .filter(|resource| resource.resource_type == ResourceType::Qemu)
        .find(|resource| resource.vmid.expect("vmid should be defined") == vmid);

    resource
        .map(|resource| resource.node.expect("node should be defined"))
        .ok_or(Problem::VMNotFound(vmid))
}
