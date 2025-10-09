use std::str::FromStr;

use auth::{OpenID, mock::WithWellKnown};
use frn_core::{identity::User, resourcemanager::Organization};
use hypervisor_connector_proxmox::mock::{
    WithClusterResourceList, WithTaskStatusReadMock, WithVMStatusStartMock,
};
use instances::{
    Instance,
    v1::{StartInstanceRequest, instances_client::InstancesClient},
};
use mock_server::MockServer;
use server::Config;
use sqlx::types::Uuid;
use tonic::{Request, metadata::MetadataValue};

#[sqlx::test(migrations = "../migrations")]
async fn test_the_start_instance_procedure_works(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange the grpc server and a client
    let mock = MockServer::new()
        .await
        .with_cluster_resource_list()
        .with_task_status_read()
        .with_vm_status_start()
        .with_well_known();
    let mock_url = mock.url();

    let organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await?;
    let instance = Instance::factory()
        .for_hypervisor_with(move |hypervisor| {
            hypervisor
                .for_default_zone()
                .for_default_organization()
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project_with(move |project| project.organization_id(organization.id))
        .distant_id("100".into())
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
    let mut request = Request::new(StartInstanceRequest {
        id: instance.id.to_string(),
    });
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", &token)).unwrap(),
    );
    let response = client.start_instance(request).await;

    // Assert the result
    assert!(response.is_ok());

    // Shutdown the server
    shutdown_tx.send(()).ok();
    Ok(())
}
