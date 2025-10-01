use auth::{OpenID, mock::WithWellKnown};
use frn_core::models::{Organization, User};
use hypervisor_connector_proxmox::mock::{
    WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock, WithVMCloneMock,
};
use instances::{
    Instance,
    v1::{CloneInstanceRequest, instances_client::InstancesClient},
};
use mock_server::MockServer;
use server::Config;
use std::str::FromStr;
use tonic::{Request, metadata::MetadataValue};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_clone_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_next_id()
        .with_cluster_resource_list()
        .with_vm_clone()
        .with_task_status_read()
        .with_well_known();
    let mock_url = mock.url();

    let organization = Organization::factory().create(&pool).await?;

    let instance = Instance::factory()
        .distant_id("100".into())
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .url(mock_url)
                .for_default_datacenter()
                .organization_id(organization.id)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .create(&pool)
        .await?;

    let user = User::factory()
        .email("wile.coyote@acme.org".to_owned())
        .create(&pool)
        .await
        .expect("could not create user");
    let token = OpenID::token(&user.email);

    let config = Config::test(&pool, &mock).await?;
    let server_url = format!("http://{}", config.addr);
    let shutdown_tx = server::serve(config).await?;
    let mut client = InstancesClient::connect(server_url).await?;

    // Act the request to the test_the_status_procedure_works
    let mut request = Request::new(CloneInstanceRequest {
        id: instance.id.to_string(),
    });
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", &token)).unwrap(),
    );

    let response = client.clone_instance(request).await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
