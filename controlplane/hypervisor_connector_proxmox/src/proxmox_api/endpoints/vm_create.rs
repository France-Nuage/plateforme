use hypervisor_connector::InstanceConfig;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};

pub async fn vm_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    options: &VMConfig,
) -> Result<ApiResponse<String>, crate::proxmox_api::problem::Problem> {
    client
        .post(format!("{}/api2/json/nodes/{}/qemu", api_url, node_id))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(options)
        .send()
        .await
        .to_api_response()
        .await
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct VMConfig {
    /// Enable/disable communication with the QEMU Guest Agent and its properties.
    pub agent: Option<String>,

    /// Specify guest boot order.
    pub boot: Option<String>,

    /// Specify custom files to replace the automatically generated ones at start.
    pub cicustom: Option<String>,

    /// Emulated CPU type.
    pub cpu: Option<String>,

    /// The number of cores per socket.
    pub cores: Option<u8>,

    /// Use volume as IDE hard disk or CD-ROM.
    pub ide2: Option<String>,

    /// Memory properties.
    pub memory: Option<u32>,

    /// Set a name for the VM. Only used on the configuration web interface.
    pub name: Option<String>,

    /// cloud-init: Sets DNS server IP address for a container.
    pub nameserver: Option<String>,

    /// Specify network devices.
    pub net0: Option<String>,

    /// Use volume as SCSI hard disk or CD-ROM.
    pub scsi0: Option<String>,

    /// SCSI controller model.
    pub scsihw: Option<String>,

    /// Create a serial device inside the VM.
    pub serial0: Option<String>,

    /// The number of CPU sockets.
    pub sockets: Option<u8>,

    /// Enable/disable Template.
    pub template: Option<bool>,

    /// Configure the VGA Hardware.
    pub vga: Option<String>,

    /// The (unique) ID of the VM.
    pub vmid: u32,
}

impl Default for VMConfig {
    fn default() -> Self {
        let snippets_storage =
            std::env::var("PROXMOX_SNIPPETS_STORAGE").unwrap_or_else(|_| String::from("CephPool"));
        let image_storage =
            std::env::var("PROXMOX_IMAGE_STORAGE").unwrap_or_else(|_| String::from("CephPool"));
        VMConfig {
            agent: Some(String::from("enabled=1")),
            boot: Some(String::from("c,order=scsi0")),
            cicustom: None,
            cpu: Some(String::from("x86-64-v2-AES")),
            cores: Some(1),
            ide2: Some(format!("{}:cloudinit", snippets_storage)),
            memory: Some(1024),
            name: None,
            nameserver: Some(String::from("1.1.1.1")),
            net0: Some(String::from("virtio,bridge=vmbr0")),
            scsi0: Some(format!(
                "{}:0,import-from=/var/lib/vz/images/0/debian-12-genericcloud-amd64-20241201-1948.qcow2,discard=on,ssd=1",
                image_storage,
            )),
            scsihw: Some(String::from("virtio-scsi-pci")),
            serial0: Some(String::from("socket")),
            sockets: Some(1),
            template: Some(false),
            vga: Some(String::from("serial0")),
            vmid: 0,
        }
    }
}

impl VMConfig {
    pub fn from_instance_config(value: InstanceConfig, vmid: u32) -> Self {
        let image_storage =
            std::env::var("PROXMOX_IMAGE_STORAGE").unwrap_or_else(|_| String::from("CephPool"));

        let volume = format!(
            "{}:0,import-from=local:0/{},discard=on,ssd=1",
            image_storage, value.disk_image
        );

        VMConfig {
            cicustom: Some(format!("user=nfs-snippets:snippets/{}", value.snippet)),
            cores: Some(value.cores),
            memory: Some(value.memory),
            name: Some(value.name),
            scsi0: Some(volume),
            vmid,
            ..Default::default()
        }
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithVMCreateMock {
        fn with_vm_create(self) -> Self;
    }

    impl WithVMCreateMock for MockServer {
        fn with_vm_create(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(r"^/api2/json/nodes/.*/qemu$".to_string()),
                )
                .with_body(r#"{"data":"UPID:pve-node1:0021B19E:02328820:67CC7B42:qmcreate:110:root@pam!api:"}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithVMCreateMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_create();
        let options = VMConfig {
            ..Default::default()
        };
        let result = vm_create(&server.url(), &client, "", "pve-node1", &options).await;

        assert!(result.is_ok());
    }
}
