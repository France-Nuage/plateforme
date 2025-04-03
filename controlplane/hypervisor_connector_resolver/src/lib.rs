use hypervisor_connector::InstanceService;

pub fn resolve(api_url: String, client: reqwest::Client) -> impl InstanceService {
    hypervisor_connector_proxmox::ProxmoxInstanceService {
        api_url,
        client,
        id: 100,
    }
}
