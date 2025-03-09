use serde::Deserialize;

mod api;
mod proxmox_cluster;
mod proxmox_instance;
mod proxmox_node;

pub use proxmox_cluster::ProxmoxCluster;
pub use proxmox_instance::ProxmoxVM;
pub use proxmox_node::ProxmoxNode;

#[derive(Debug, Deserialize)]
pub struct ProxmoxResponse<T> {
    data: T,
}

#[cfg(test)]
pub mod tests {
    pub struct MockServer {
        pub mocks: Vec<mockito::Mock>,
        server: mockito::ServerGuard,
    }

    impl MockServer {
        pub async fn new() -> Self {
            MockServer {
                server: mockito::Server::new_async().await,
                mocks: vec![],
            }
        }

        /// Expose a mock for the `/api2/json/nodes/{node}/qemu/{vmid}/status/current` endpoint.
        pub fn with_qemu_status_current(mut self) -> Self {
            // Create the mock
            let mock = self.server
            .mock("GET", mockito::Matcher::Regex(r"^/api2/json/nodes/pve-node1/qemu/\d+/status/current$".to_string()))
            .with_body(r#"{"data":{"serial":1,"ballooninfo":{"total_mem":1020547072,"major_page_faults":1005,"mem_swapped_in":0,"mem_swapped_out":0,"last_update":1741275609,"actual":1073741824,"max_mem":1073741824,"minor_page_faults":2919589,"free_mem":343924736},"running-machine":"pc-i440fx-9.0+pve0","maxmem":1073741824,"agent":1,"name":"debian12-empty-for-rce-testing","cpus":1,"ha":{"managed":0},"cpu":0.00681408524476239,"diskread":313953890,"running-qemu":"9.0.2","mem":676622336,"netout":170441,"maxdisk":10737418240,"netin":87942820,"pid":10189,"vmid":105,"uptime":189524,"disk":0,"balloon":1073741824,"blockstat":{"scsi0":{"rd_bytes":313609728,"idle_time_ns":26650143884,"zone_append_merged":0,"wr_highest_offset":10737418240,"invalid_unmap_operations":0,"zone_append_operations":0,"invalid_zone_append_operations":0,"invalid_rd_operations":0,"rd_merged":0,"invalid_wr_operations":0,"unmap_operations":1378,"wr_operations":24645,"failed_zone_append_operations":0,"timed_stats":[],"unmap_total_time_ns":11003682769,"wr_total_time_ns":1749888914003,"flush_operations":13945,"flush_total_time_ns":23672268419,"failed_unmap_operations":0,"wr_bytes":2054091264,"zone_append_total_time_ns":0,"failed_wr_operations":0,"rd_total_time_ns":26768833060,"account_failed":true,"invalid_flush_operations":0,"unmap_bytes":7445999616,"zone_append_bytes":0,"failed_flush_operations":0,"wr_merged":0,"failed_rd_operations":0,"account_invalid":true,"unmap_merged":0,"rd_operations":12385},"ide2":{"failed_wr_operations":0,"zone_append_total_time_ns":0,"wr_bytes":0,"flush_total_time_ns":0,"failed_unmap_operations":0,"flush_operations":0,"rd_operations":92,"unmap_merged":0,"account_invalid":true,"wr_merged":0,"failed_flush_operations":0,"failed_rd_operations":0,"zone_append_bytes":0,"account_failed":true,"invalid_flush_operations":0,"unmap_bytes":0,"rd_total_time_ns":146108582,"invalid_unmap_operations":0,"wr_highest_offset":0,"zone_append_merged":0,"idle_time_ns":189500418903491,"rd_bytes":344162,"wr_total_time_ns":0,"unmap_total_time_ns":0,"timed_stats":[],"failed_zone_append_operations":0,"wr_operations":0,"invalid_wr_operations":0,"rd_merged":0,"unmap_operations":0,"invalid_rd_operations":0,"zone_append_operations":0,"invalid_zone_append_operations":0}},"nics":{"tap105i0":{"netin":87942820,"netout":170441}},"freemem":343924736,"clipboard":null,"status":"running","diskwrite":2054091264,"proxmox-support":{"backup-fleecing":true,"pbs-library-version":"1.4.1 (UNKNOWN)","query-bitmap-info":true,"backup-max-workers":true,"pbs-masterkey":true,"pbs-dirty-bitmap-savevm":true,"pbs-dirty-bitmap":true,"pbs-dirty-bitmap-migration":true},"qmpstatus":"running"}}"#)
            .create();

            // Register the mock so it is not dropped
            self.mocks.push(mock);

            self
        }

        /// Expose a mock for the `/api2/json/nodes/{node}/qemu/{vmid}/status/stop` endpoint.
        pub fn with_qemu_status_start(mut self) -> Self {
            let mock = self.server.mock("POST", mockito::Matcher::Regex(r"^/api2/json/nodes/pve-node1/qemu/\d+/status/start$".to_string()))
                .with_body(r#"{"data": "UPID:pve-node1:00194D13:01A5F253:67CB135C:qmstop:105:root@pam!api:"}"#)
                .create();

            // Register the mock so it is not dropped
            self.mocks.push(mock);

            self
        }

        /// Expose a mock for the `/api2/json/nodes/{node}/qemu/{vmid}/status/stop` endpoint.
        pub fn with_qemu_status_stop(mut self) -> Self {
            let mock = self.server.mock("POST", mockito::Matcher::Regex(r"^/api2/json/nodes/pve-node1/qemu/\d+/status/stop$".to_string()))
                .with_body(r#"{"data": "UPID:pve-node1:00194D13:01A5F253:67CB135C:qmstop:105:root@pam!api:"}"#)
                .create();

            // Register the mock so it is not dropped
            self.mocks.push(mock);

            self
        }

        /// The URL of the mock server (including the protocol).
        pub fn url(&self) -> String {
            self.server.url()
        }
    }
}
