use serde::Deserialize;

use crate::proxmox_api::{
    Problem,
    api_response::{ApiResponse, ApiResponseExt},
};

pub async fn task_status_read(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    task_id: &str,
) -> Result<ApiResponse<TaskStatusResponse>, Problem> {
    client
        .get(format!(
            "{}/api2/json/nodes/{}/tasks/{}/status",
            api_url, node_id, task_id
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Deserialize)]
pub struct TaskStatusResponse {
    pub id: String,
    pub node: String,
    pub pid: u32,
    pub pstart: u32,
    pub starttime: u64,
    pub status: TaskStatus,
    #[serde(rename = "type")]
    pub task_type: String,
    pub upid: String,
    pub user: String,
    #[serde(rename = "exitStatus")]
    pub exit_status: Option<TaskExitStatus>,
}

#[derive(Debug, Deserialize)]
pub enum TaskExitStatus {
    #[serde(alias = "OK")]
    Ok,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Running,
    Stopped,
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithTaskStatusReadMock {
        fn with_task_status_read(self) -> Self;
    }

    impl WithTaskStatusReadMock for MockServer {
        fn with_task_status_read(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/tasks/.*/status$".to_string()),
                )
                .with_body(r#"{"data":{"node":"pvedev01-dc03","type":"qmcreate","exitstatus":"OK","pstart":8526388,"tokenid":"robin","status":"stopped","starttime":1745086489,"pid":396816,"id":"666","user":"root@pam","upid":"UPID:pvedev01-dc03:00060E10:00821A34:6803E819:qmcreate:666:root@pam!robin:"}}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithTaskStatusReadMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_task_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_task_status_read();
        let result = task_status_read(&server.url(), &client, "", "pve-node1", "foobar").await;

        assert!(result.is_ok());
    }
}
