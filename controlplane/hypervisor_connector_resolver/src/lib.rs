use hypervisor_connector::InstanceService;

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

pub fn resolve_model(model: hypervisors::Model) -> impl InstanceService {
    hypervisor_connector_proxmox::ProxmoxInstanceService {
        api_url: model.url,
        client: reqwest::Client::new(),
        authorization: model.authentication_token,
        id: 100,
    }
}
