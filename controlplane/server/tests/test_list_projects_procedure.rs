use auth::JwkValidator;
use hypervisor_connector_proxmox::mock::MockServer;
use resources::{
    projects::Project,
    v1::{ListProjectsRequest, resources_client::ResourcesClient},
};
use server::Config;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_list_projects_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the test
    let mock = MockServer::new().await;
    let oidc_url = mock.url();

    let config = Config::new(
        pool.clone(),
        JwkValidator::from_oidc_discovery(&oidc_url).await?,
    );
    Project::factory()
        .for_organization_with(|organization| organization)
        .create(&pool)
        .await?;

    let addr = format!("http://{}", config.addr);
    let shutdown_tx = server::serve_with_tx(config).await?;
    let mut client = ResourcesClient::connect(addr).await?;

    // Act the request to the test_the_status_procedure_works
    let response = client.list_projects(ListProjectsRequest::default()).await;

    // Assert the result
    assert!(response.is_ok());
    assert_eq!(response.unwrap().into_inner().projects.len(), 1);

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
