mod api;
mod models;

pub async fn run(client: &reqwest::Client) -> Result<(), reqwest::Error> {
    // Create a query for requesting all instances
    let query = api::ComputeInstanceIndexQuery {
        ..Default::default()
    };

    // Get all instances from the API
    let instances = api::pagination::unfold_api_list(client, query, api::list_compute_instances)
        .await
        .unwrap();

    // For every instance check the status and update if need be
    for instance in instances {
        let query = api::InstanceHypervisorStatusQuery::from_instance(&instance);
        let result = api::get_instance_hypervisor_status(client, query)
            .await
            .unwrap();

        if result.status != instance.status {
            log::info!(
                "Updating instance {} status: {:?} => {:?}",
                instance.id,
                instance.status,
                result.status
            );
            let query = api::UpdateComputeInstanceQuery {
                instance_id: instance.id,
                status: result.status,
            };
            api::update_compute_instance(client, query).await?;
        }
    }
    Ok(())
}
