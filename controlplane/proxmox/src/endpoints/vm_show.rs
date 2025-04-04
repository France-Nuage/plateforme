use serde::Deserialize;

use crate::api_response::{ApiResponse, ApiResponseExt};

pub async fn vm_show(
    api_url: &str,
    client: &reqwest::Client,
    node_id: &str,
) -> Result<ApiResponse<Vec<VMInfo>>, crate::problem::Problem> {
    client
        .get(format!("{}/api2/json/nodes/{}/qemu/{}/rrddata", api_url, node_id, vm_id))
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

    pub trait WithVMShowMock {
        fn with_vm_show(self) -> Self;
    }

    impl WithVMShowMock for MockServer {
        fn with_vm_show(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.*/qemu/\d+/rrddata$".to_string(),
                    ),
                )
                .with_body(r#"{"data":{"netin":0,"serial":1,"maxdisk":10737418240,"mem":0,"netout":0,"disk":0,"uptime":0,"maxmem":1073741824,"vmid":1500,"name":"debian12-docker-empty-template","cpus":1,"cpu":0,"template":1,"diskwrite":0,"status":"stopped","diskread":0}}"#)
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
    async fn test_vm_show() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_list();
        let result = vm_show(&server.url(), &client, "pve-node1", 100).await;
        println!("{:?}", result);

        assert!(result.is_ok());
    }
}
