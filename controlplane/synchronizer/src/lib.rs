use frn_core::{authorization::AuthorizationServer, identity::ServiceAccount};
use instances::InstancesService;
use std::error::Error;
use tracing::error;

pub async fn synchronize<Auth: AuthorizationServer>(
    service: &mut InstancesService<Auth>,
) -> Result<(), Box<dyn Error>> {
    let principal = ServiceAccount::default();
    service.sync(&principal).await?;
    Ok(())
}

pub async fn heartbeat(client: &reqwest::Client, url: &Option<String>) {
    if let Some(url) = url
        && let Err(e) = client.get(url).send().await
    {
        error!(error = %e, "Failed to ping heartbeat");
    }
}
