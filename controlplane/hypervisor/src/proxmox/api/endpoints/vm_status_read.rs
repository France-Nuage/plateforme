use crate::proxmox::api::Error;
use crate::proxmox::api::ResourceStatus;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::Deserialize;

pub async fn vm_status_read(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vm_id: u32,
) -> Result<ApiResponse<VMStatusResponse>, Error> {
    client
        .get(format!(
            "{}/api2/json/nodes/{}/qemu/{}/status/current",
            api_url, node_id, vm_id
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct VMStatusResponse {
    pub status: ResourceStatus,
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithVMStatusReadMock {
        fn with_vm_status_read(self) -> Self;
    }

    impl WithVMStatusReadMock for MockServer {
        fn with_vm_status_read(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.*/qemu/\d+/status/current$".to_string(),
                    ),
                )
                .with_body(r#"{"data":{"proxmox-support":{"query-bitmap-info":true,"backup-max-workers":true,"pbs-dirty-bitmap-savevm":true,"pbs-masterkey":true,"pbs-dirty-bitmap":true,"pbs-dirty-bitmap-migration":true,"backup-fleecing":true,"pbs-library-version":"1.4.1 (UNKNOWN)"},"qmpstatus":"running","diskwrite":213608448,"status":"running","clipboard":null,"freemem":599695360,"blockstat":{"scsi0":{"flush_total_time_ns":940486369,"failed_unmap_operations":0,"flush_operations":2905,"failed_wr_operations":0,"zone_append_total_time_ns":0,"wr_bytes":213608448,"zone_append_bytes":0,"rd_total_time_ns":15633954007,"invalid_flush_operations":0,"account_failed":true,"unmap_bytes":284041216,"rd_operations":6523,"unmap_merged":0,"account_invalid":true,"wr_merged":0,"failed_flush_operations":0,"failed_rd_operations":0,"idle_time_ns":27695724209,"zone_append_merged":0,"rd_bytes":249591296,"invalid_unmap_operations":0,"wr_highest_offset":8724213760,"wr_operations":5467,"failed_zone_append_operations":0,"unmap_total_time_ns":184655456,"timed_stats":[],"invalid_zone_append_operations":0,"zone_append_operations":0,"rd_merged":0,"invalid_wr_operations":0,"unmap_operations":102,"invalid_rd_operations":0,"wr_total_time_ns":92538702762},"ide2":{"invalid_unmap_operations":0,"wr_highest_offset":0,"idle_time_ns":91887204234178,"zone_append_merged":0,"rd_bytes":344162,"wr_total_time_ns":0,"failed_zone_append_operations":0,"wr_operations":0,"timed_stats":[],"unmap_total_time_ns":0,"zone_append_operations":0,"invalid_zone_append_operations":0,"invalid_rd_operations":0,"invalid_wr_operations":0,"rd_merged":0,"unmap_operations":0,"failed_wr_operations":0,"wr_bytes":0,"zone_append_total_time_ns":0,"flush_total_time_ns":0,"failed_unmap_operations":0,"flush_operations":0,"unmap_merged":0,"rd_operations":92,"wr_merged":0,"failed_rd_operations":0,"failed_flush_operations":0,"account_invalid":true,"zone_append_bytes":0,"rd_total_time_ns":117227368,"account_failed":true,"unmap_bytes":0,"invalid_flush_operations":0}},"nics":{"tap105i0":{"netout":26496,"netin":1880206}},"balloon":1073741824,"disk":0,"uptime":91908,"vmid":105,"pid":1661179,"netin":1880206,"maxdisk":10737418240,"running-qemu":"9.0.2","mem":420847616,"netout":26496,"diskread":249935458,"cpu":0.00592883645257438,"ha":{"managed":0},"cpus":1,"name":"debian12-empty-for-rce-testing","agent":1,"maxmem":1073741824,"running-machine":"pc-i440fx-9.0+pve0","ballooninfo":{"last_update":1741454419,"actual":1073741824,"mem_swapped_out":0,"major_page_faults":629,"mem_swapped_in":0,"total_mem":1020542976,"free_mem":599695360,"minor_page_faults":322701,"max_mem":1073741824},"serial":1}}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithVMStatusReadMock;
    use super::*;
    use crate::proxmox::api::api_response::ApiResponse;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_vm_status_read() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_status_read();
        let result = vm_status_read(&server.url(), &client, "", "pve-node1", 100).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ApiResponse {
                data: VMStatusResponse {
                    status: ResourceStatus::Running
                }
            }
        );
    }
}
