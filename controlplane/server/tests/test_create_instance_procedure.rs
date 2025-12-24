use crate::common::{Api, OnBehalfOf};
use fabrique::Persistable;
use frn_core::compute::{Hypervisor, Instance};
use frn_core::network::{VNet, VPC};
use frn_core::resourcemanager::{DEFAULT_PROJECT_NAME, Organization, Project};
use frn_rpc::v1::compute::{CreateInstanceRequest, CreateInstanceResponse};
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_create_instance_procedure_works(pool: sqlx::PgPool) {
    // Arrange a test api and the required data
    let mut api = Api::start(&pool).await.expect("count not start api");
    let organization = Organization::factory()
        .create(&pool)
        .await
        .expect("could not create organization");

    let hypervisor = Hypervisor::factory()
        .url(api.mock_server.url())
        .for_zone(|zone| zone)
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create hypervisor");

    let project = Project::factory()
        .name(DEFAULT_PROJECT_NAME.into())
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create project");

    // Create VPC and VNet for network configuration
    let vpc = VPC::factory()
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create vpc");

    let vnet = VNet::factory()
        .vpc_id(vpc.id)
        .create(&pool)
        .await
        .expect("could not create vnet");

    // Act the call to the create rpc
    let request = Request::new(CreateInstanceRequest {
        image: String::from("debian.qcow2"),
        cpu_cores: 1,
        disk_bytes: 10737418240,
        memory_bytes: 536870912,
        name: String::from("acme-mgs"),
        project_id: project.id.to_string(),
        snippet: Some(String::from("acme-snippet.yaml")),
        vpc_id: vpc.id.to_string(),
        vnet_id: vnet.id.to_string(),
        requested_ip: None,
        security_group_ids: vec![],
    })
    .on_behalf_of(&api.service_account);

    // Assert the result
    let result = api.compute.instances.create(request).await;
    assert!(result.is_ok());
    let instances = Instance::all(&pool)
        .await
        .expect("could not fetch instances");
    let instance = &instances[0];
    assert_eq!(
        result.unwrap().into_inner(),
        CreateInstanceResponse {
            instance: Some(frn_rpc::v1::compute::Instance {
                id: instance.id.to_string(),
                max_cpu_cores: 1,
                max_memory_bytes: 536870912,
                name: "acme-mgs".to_owned(),
                hypervisor_id: hypervisor.id.to_string(),
                project_id: project.id.to_string(),
                created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    instance.created_at
                ))),
                updated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    instance.updated_at
                ))),
                ..Default::default()
            })
        }
    );
}
