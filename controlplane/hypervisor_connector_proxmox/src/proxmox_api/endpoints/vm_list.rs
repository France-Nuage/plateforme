use serde::Deserialize;

use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};

pub async fn vm_list(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
) -> Result<ApiResponse<Vec<VMInfo>>, crate::proxmox_api::Problem> {
    client
        .get(format!("{}/api2/json/nodes/{}/qemu", api_url, node_id))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Deserialize)]
pub struct VMInfo {}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::mock::MockServer;

    pub trait WithVMListMock {
        fn with_vm_list(self) -> Self;
    }

    impl WithVMListMock for MockServer {
        fn with_vm_list(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/qemu$".to_string()),
                )
                .with_body(r#"{"data":[{"netin":0,"serial":1,"maxdisk":10737418240,"mem":0,"netout":0,"disk":0,"uptime":0,"maxmem":1073741824,"vmid":1500,"name":"debian12-docker-empty-template","cpus":1,"cpu":0,"template":1,"diskwrite":0,"status":"stopped","diskread":0},{"maxmem":1073741824,"uptime":91822,"disk":0,"vmid":105,"name":"debian12-empty-for-rce-testing","pid":1661179,"netin":1879246,"mem":243718053,"netout":26496,"serial":1,"maxdisk":10737418240,"diskwrite":0,"diskread":0,"status":"running","cpus":1,"cpu":0.00608236152985228},{"status":"stopped","diskread":0,"diskwrite":0,"cpus":1,"cpu":0,"vmid":110,"disk":0,"maxmem":1073741824,"uptime":0,"name":"VM 110","maxdisk":0,"netout":0,"mem":0,"netin":0}]}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockServer, WithVMListMock};

    #[tokio::test]
    async fn test_vm_list() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_list();
        let result = vm_list(&server.url(), &client, "", "pve-node1").await;
        println!("{:?}", result);

        assert!(result.is_ok());
    }
}
