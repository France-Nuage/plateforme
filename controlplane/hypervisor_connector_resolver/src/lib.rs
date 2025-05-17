use hypervisor_connector::InstanceService;
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
    }
}

pub fn resolve_for_hypervisor(hypervisor: &Hypervisor) -> impl InstanceService {
    hypervisor_connector_proxmox::ProxmoxInstanceService {
        api_url: hypervisor.url.clone(),
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        authorization: hypervisor.authorization_token.clone(),
    }
}
