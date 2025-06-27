use std::fmt::Display;

use serde::Deserialize;

use crate::proxmox_api::{
    Problem, VMStatus,
    api_response::{ApiResponse, ApiResponseExt},
};

pub async fn cluster_resources_list(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    resource_type: &str,
) -> Result<ApiResponse<Vec<Resource>>, crate::proxmox_api::Problem> {
    client
        .get(format!(
            "{}/api2/json/cluster/resources?type={}",
            api_url, resource_type
        ))
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

    /// Used disk space in bytes (for type 'storage'), used root image space for VMs (for types 'qemu' and 'lxc').
    pub disk: Option<u64>,

    /// Number of available CPUs (for types 'node', 'qemu' and 'lxc').
    pub maxcpu: Option<u8>,

    /// Storage size in bytes (for type 'storage'), root image size for VMs (for types 'qemu' and 'lxc').
    pub maxdisk: Option<u64>,

    /// Number of available memory in bytes (for types 'node', 'qemu' and 'lxc').
    pub maxmem: Option<u64>,

    /// Used memory in bytes (for types 'node', 'qemu' and 'lxc').
    pub mem: Option<u64>,

    /// Name of the resource.
    pub name: Option<String>,

    /// The node holding the resource,
    pub node: Option<String>,

    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: ResourceType,

    /// Resource type dependent status.
    pub status: Option<VMStatus>,

    /// The numerical vmid (for types 'qemu' and 'lxc').
    pub vmid: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Node,
    Storage,
    Pool,
    Qemu,
    Lxc,
    Openvz,
    Sdn,
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResourceType::Node => "node",
            ResourceType::Storage => "storage",
            ResourceType::Pool => "pool",
            ResourceType::Qemu => "qemu",
            ResourceType::Lxc => "lxc",
            ResourceType::Openvz => "openvz",
            ResourceType::Sdn => "sdn",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<Resource> for hypervisor_connector::InstanceInfo {
    type Error = Problem;

    fn try_from(value: Resource) -> Result<Self, Self::Error> {
        let required_field = |field: &str| Problem::ResourceMissingField {
            id: value
                .vmid
                .map(|id| id.to_string())
                .unwrap_or("unknown".to_owned()),
            resource_type: value.resource_type.clone(),
            field: field.to_owned(),
        };
        let info = hypervisor_connector::InstanceInfo {
            cpu_usage_percent: value.cpu.ok_or(required_field("cpu_usage_percent"))?,
            disk_usage_bytes: value.disk.ok_or(required_field("disk_usage_bytes"))?,
            id: value.vmid.ok_or(required_field("id"))?.to_string(),
            max_cpu_cores: value.maxcpu.ok_or(required_field("max_cpu_cores"))? as u32,
            max_disk_bytes: value.maxdisk.ok_or(required_field("max_disk_bytes"))?,
            max_memory_bytes: value.maxmem.ok_or(required_field("max_memory_bytes"))?,
            memory_usage_bytes: value.mem.ok_or(required_field("memory_usage_bytes"))?,
            name: value.name.unwrap_or_else(|| String::from("unnamed")),
            status: value.status.expect("no status in response").into(),
        };

        Ok(info)
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
                    mockito::Matcher::Regex(r"^/api2/json/cluster/resources\?type=.*$".to_string()),
                )
                .with_body(r#"{"data":[{"status":"running","maxmem":4294967296,"hastate":"started","diskread":1441248256,"diskwrite":218681344,"maxcpu":1,"netout":33288,"id":"qemu/100","mem":1395277824,"cpu":0.115798987285604,"template":0,"pool":"CephPool","vmid":100,"disk":0,"node":"pve-node1","uptime":20961,"type":"qemu","netin":321018,"maxdisk":53687091200,"name":"proxmox-dev"}]}"#)
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
        let result = cluster_resources_list(&server.url(), &client, "", "vm").await;

        assert!(result.is_ok());
        let resources = result.unwrap().data;
        assert_eq!(
            resources,
            vec![Resource {
                cpu: Some(0.11579899),
                disk: Some(0),
                maxcpu: Some(1),
                maxdisk: Some(53687091200),
                maxmem: Some(4294967296),
                mem: Some(1395277824),
                name: Some(String::from("proxmox-dev")),
                node: Some(String::from("pve-node1")),
                resource_type: ResourceType::Qemu,
                status: Some(VMStatus::Running),
                vmid: Some(100),
            }]
        );
    }
}
