use hypervisor_connector::InstanceService;
use hypervisor_connector_proxmox::ProxmoxInstanceService;

pub fn get_instance_service(api_url: String, client: reqwest::Client) -> impl InstanceService {
    ProxmoxInstanceService {
        api_url,
        client,
        id: 100,
    }
}
