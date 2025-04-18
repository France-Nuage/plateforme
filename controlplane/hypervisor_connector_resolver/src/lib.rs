use hypervisor_connector::{InstanceInfo, InstanceService};
use hypervisors::Hypervisor;

pub fn resolve(
    api_url: String,
    client: reqwest::Client,
    authorization: String,
) -> impl InstanceService {
    hypervisor_connector_proxmox::ProxmoxInstanceService {
        api_url,
        authorization,
        client,
        id: 100,
    }
}

pub fn resolve_for_hypervisor(hypervisor: &Hypervisor, id: u32) -> impl InstanceService {
    hypervisor_connector_proxmox::ProxmoxInstanceService {
        api_url: hypervisor.url.clone(),
        client: reqwest::Client::new(),
        authorization: hypervisor.authorization_token.clone(),
        id,
    }
}
