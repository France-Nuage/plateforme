use instances::InstancesService;
use std::error::Error;
use tracing::{error, instrument};

#[instrument(skip(service))]
pub async fn synchronize(service: &InstancesService) -> Result<(), Box<dyn Error>> {
    service.sync().await?;
    Ok(())
}

pub async fn heartbeat(client: &reqwest::Client, url: &Option<String>) {
    if let Some(url) = url {
        if let Err(e) = client.get(url).send().await {
            error!(error = %e, "Failed to ping heartbeat");
        }
    }
}
