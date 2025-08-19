use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};
use serde::{Deserialize, Deserializer};
use std::{net::Ipv4Addr, str};

pub async fn vm_config_read(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vmid: u32,
) -> Result<ApiResponse<VMConfig>, crate::proxmox_api::problem::Problem> {
    client
        .get(format!(
            "{}/api2/json/nodes/{}/qemu/{}/config",
            api_url, node_id, vmid
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct VMConfig {
    #[serde(deserialize_with = "deserialize_ipconfig", default)]
    pub ipconfig0: Option<IpConfig>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct IpConfig {
    pub ip: Ipv4Addr,
    pub cidr: u8,
    pub gateway: Ipv4Addr,
}

fn deserialize_ipconfig<'de, D>(deserializer: D) -> Result<Option<IpConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => parse_ip_config(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

fn parse_ip_config(s: &str) -> Result<IpConfig, String> {
    let parts: Vec<&str> = s.split(',').collect();

    let ip_part = parts[0].strip_prefix("ip=").ok_or("Missing 'ip=' prefix")?;
    let gw_part = parts[1].strip_prefix("gw=").ok_or("Missing 'gw=' prefix")?;

    let (ip_str, cidr_str) = ip_part.split_once('/').ok_or("Missing CIDR notation")?;

    let ip = ip_str
        .parse::<Ipv4Addr>()
        .map_err(|e| format!("Invalid IP: {}", e))?;
    let cidr = cidr_str
        .parse::<u8>()
        .map_err(|e| format!("Invalid CIDR: {}", e))?;
    let gateway = gw_part
        .parse::<Ipv4Addr>()
        .map_err(|e| format!("Invalid gateway: {}", e))?;

    Ok(IpConfig { ip, cidr, gateway })
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::mock::MockServer;

    pub trait WithVMConfigMock {
        fn with_vm_config(self) -> Self;
    }

    impl WithVMConfigMock for MockServer {
        fn with_vm_config(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/qemu/.*/config$".to_string()),
                )
                .with_body(r#"{"data":{"scsihw":"virtio-scsi-pci","cores":1,"meta":"creation-qemu=9.2.0,ctime=1752413379","net0":"virtio=BC:24:11:20:4E:3F,bridge=vmbr0","cpu":"x86-64-v2-AES","memory":"1024","name":"empty","vga":"serial0","bootdisk":"scsi0","agent":"enabled=1","scsi0":"ceph-pool-nvme-01:vm-555-disk-0,discard=on,size=50G,ssd=1","ide2":"ceph-pool-nvme-01:vm-555-cloudinit,media=cdrom","onboot":1,"numa":0,"ipconfig0":"ip=10.2.16.80/21,gw=10.2.16.1","nameserver":"8.8.8.8","cicustom":"user=nfs-snippets:snippets/ci-custom-empty-snippet.yaml","boot":"c","digest":"0d7b4aefa97d9a0dacfcfb0016fa1e614bc56cc6","smbios1":"uuid=6b03e691-fe59-451d-9007-b13629771ad8","vmgenid":"3c8a4d6f-7cd0-423d-903f-e1493b8d4a3a","sockets":1,"serial0":"socket"}}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::mock::{MockServer, WithVMConfigMock};

    #[tokio::test]
    async fn test_vm_config_read() {
        // Arrange a mock server
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_config();

        // Act the call to the vm_config_read method
        let result = vm_config_read(&server.url(), &client, "", "pve-node1", 100).await;

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().data,
            VMConfig {
                ipconfig0: Some(IpConfig {
                    ip: Ipv4Addr::from_str("10.2.16.80").unwrap(),
                    cidr: 21,
                    gateway: Ipv4Addr::from_str("10.2.16.1").unwrap()
                })
            }
        )
    }

    #[tokio::test]
    async fn test_vm_config_handles_none_ipconfig0() {}
}
