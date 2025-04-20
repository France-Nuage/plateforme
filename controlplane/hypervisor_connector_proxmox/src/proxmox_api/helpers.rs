use std::time::Duration;

use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};

use super::{
    Problem,
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
