use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::api_response::{ApiResponse, ApiResponseExt};

pub async fn vm_create(
    api_url: &str,
    client: &reqwest::Client,
    node_id: &str,
    options: &VMConfig<'_>,
) -> Result<ApiResponse<String>, crate::error::Error> {
    client
        .post(format!("{}/api2/json/nodes/{}/qemu", api_url, node_id))
        .json(options)
        .send()
        .await
        .to_api_response()
        .await
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct VMConfig<'a> {
    /// Enable/disable communication with the QEMU Guest Agent and its properties.
    agent: Option<&'a str>,

    /// Specify guest boot order.
    boot: Option<&'a str>,

    /// Emulated CPU type.
    cpu: Option<&'a str>,

    /// The number of cores per socket.
    cores: Option<u8>,

    /// Use volume as IDE hard disk or CD-ROM.
    ide2: Option<&'a str>,

    /// Memory properties.
    memory: Option<u32>,

    /// Set a name for the VM. Only used on the configuration web interface.
    name: Option<&'a str>,

    /// cloud-init: Sets DNS server IP address for a container.
    nameserver: Option<&'a str>,

    /// Specify network devices.
    net0: Option<&'a str>,

    /// Use volume as SCSI hard disk or CD-ROM.
    scsi0: Option<&'a str>,

    /// SCSI controller model.
    scsihw: Option<&'a str>,

    /// Create a serial device inside the VM.
    serial0: Option<&'a str>,

    /// The number of CPU sockets.
    sockets: Option<u8>,

    /// Enable/disable Template.
    template: Option<bool>,

    /// Configure the VGA Hardware.
    vga: Option<&'a str>,

    /// The (unique) ID of the VM.
    vmid: u32,
}

impl<'a> std::convert::From<&'a hypervisor::InstanceConfig<'a>> for VMConfig<'a> {
    fn from(config: &'a hypervisor::InstanceConfig) -> Self {
        VMConfig {
            name: Some(config.name),
            vmid: config.id.parse().unwrap(),
            ..Default::default()
        }
    }
}

impl Default for VMConfig<'_> {
    fn default() -> Self {
        VMConfig {
            agent: Some("enabled=1"),
            boot: Some("c,order=scsi0"),
            cpu: Some("x86-64-v2-AES"),
            cores: Some(1),
            ide2: Some("ceph-pool-nvme-01:cloudinit"),
            memory: Some(1024),
            name: None,
            nameserver: Some("1.1.1.1"),
            net0: Some("virtio,bridge=vmbr0"),
            scsi0: Some(
                "CephPool:0,import-from=/var/lib/vz/images/0/debian-12-genericcloud-amd64-20241201-1948.qcow2,discard=on,ssd=1",
            ),
            scsihw: Some("virtio-scsi-pci"),
            serial0: Some("socket"),
            sockets: Some(1),
            template: Some(false),
            vga: Some("serial0"),
            vmid: 0,
        }
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::mock::MockServer;

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
    use super::*;
    use crate::mock::{MockServer, WithVMCreateMock};

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_create();
        let options = VMConfig {
            ..Default::default()
        };
        let result = vm_create(&server.url(), &client, "pve-node1", &options).await;

        assert!(result.is_ok());
    }
}
