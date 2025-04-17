use serde::Deserialize;

use crate::proxmox_api::{
    VMStatus,
    api_response::{ApiResponse, ApiResponseExt},
};

pub async fn cluster_resources_list(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
) -> Result<ApiResponse<Vec<Resource>>, crate::proxmox_api::Problem> {
    client
        .get(format!("{}/api2/json/cluster/resources?type=vm", api_url))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Resource {
    /// CPU utilization (for types 'node', 'qemu' and 'lxc').
    pub cpu: Option<f32>,

    /// Number of available CPUs (for types 'node', 'qemu' and 'lxc').
    pub maxcpu: Option<u8>,

    /// Number of available memory in bytes (for types 'node', 'qemu' and 'lxc').
    pub maxmem: Option<u64>,

    /// Used memory in bytes (for types 'node', 'qemu' and 'lxc').
    pub mem: Option<u64>,

    /// Name of the resource.
    pub name: Option<String>,

    /// Resource type dependent status.
    pub status: Option<VMStatus>,

    /// The numerical vmid (for types 'qemu' and 'lxc').
    pub vmid: Option<u32>,
}

impl From<Resource> for hypervisor_connector::InstanceInfo {
    fn from(value: Resource) -> Self {
        hypervisor_connector::InstanceInfo {
            cpu_usage_percent: value.cpu.unwrap(),
            id: value.vmid.unwrap().to_string(),
            max_cpu_cores: value.maxcpu.unwrap() as u32,
            max_memory_bytes: value.maxmem.unwrap(),
            memory_usage_bytes: value.mem.unwrap(),
            name: value.name.unwrap_or_else(|| String::from("unnamed")),
            status: value.status.expect("no status in response").into(),
        }
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::mock::MockServer;

    pub trait WithClusterResourceList {
        fn with_cluster_resource_list(self) -> Self;
    }

    impl WithClusterResourceList for MockServer {
        fn with_cluster_resource_list(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api2/json/cluster/resources\?type=vm$".to_string()),
                )
                .with_body(r#"{"data":[{"status":"running","maxmem":4294967296,"hastate":"started","diskread":1441248256,"diskwrite":218681344,"maxcpu":1,"netout":33288,"id":"qemu/100","mem":1395277824,"cpu":0.115798987285604,"template":0,"pool":"CephPool","vmid":100,"disk":0,"node":"pve-node3","uptime":20961,"type":"qemu","netin":321018,"maxdisk":53687091200,"name":"proxmox-dev"}]}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockServer, WithClusterResourceList};

    #[tokio::test]
    async fn test_cluster_resource_list() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_cluster_resource_list();
        let result = cluster_resources_list(&server.url(), &client, "").await;

        assert!(result.is_ok());
        let resources = result.unwrap().data;
        assert_eq!(
            resources,
            vec![Resource {
                cpu: Some(0.11579899),
                maxcpu: Some(1),
                maxmem: Some(4294967296),
                mem: Some(1395277824),
                name: Some(String::from("proxmox-dev")),
                status: Some(VMStatus::Running),
                vmid: Some(100),
            }]
        );
    }
}
