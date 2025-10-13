pub fn resolve(url: String, token: String) -> impl crate::instance::Instances {
    crate::proxmox::instance::ProxmoxInstanceService {
        api_url: url,
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        authorization: token,
    }
}
