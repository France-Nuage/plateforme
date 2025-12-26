use crate::common::{Api, OnBehalfOf};
use fabrique::Persistable;
use frn_core::compute::{Hypervisor, Instance};
use frn_core::network::{AllocationType, IPAllocation, VNet, VPC};
use frn_core::resourcemanager::{DEFAULT_PROJECT_NAME, Organization, Project};
use frn_rpc::v1::compute::CreateInstanceRequest;
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
        .mtu(1450)
        .vxlan_tag(100)
        .create(&pool)
        .await
        .expect("could not create vpc");

    let vnet = VNet::factory()
        .vpc_id(vpc.id)
        .subnet("10.0.1.0/24".to_string())
        .gateway("10.0.1.1".to_string())
        .create(&pool)
        .await
        .expect("could not create vnet");

    // Reserve gateway IP in IPAM (normally done by VNets::create service)
    IPAllocation::factory()
        .vnet_id(vnet.id)
        .address("10.0.1.1".to_string())
        .allocation_type(AllocationType::Gateway.to_string())
        .hostname(Some("gateway".to_string()))
        .create(&pool)
        .await
        .expect("could not reserve gateway IP");

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
    assert!(result.is_ok(), "create instance failed: {:?}", result.err());
    let instances = Instance::all(&pool)
        .await
        .expect("could not fetch instances");
    let instance = &instances[0];
    let response = result.unwrap().into_inner();
    let response_instance = response.instance.expect("response should contain instance");

    // Verify basic instance properties
    assert_eq!(response_instance.id, instance.id.to_string());
    assert_eq!(response_instance.name, "acme-mgs");
    assert_eq!(response_instance.max_cpu_cores, 1);
    assert_eq!(response_instance.max_memory_bytes, 536870912);
    assert_eq!(response_instance.hypervisor_id, hypervisor.id.to_string());
    assert_eq!(response_instance.project_id, project.id.to_string());

    // Verify network configuration from VPC/VNet
    assert_eq!(response_instance.ip_v4, "10.0.1.2"); // First available IP after gateway
    assert!(
        response_instance.mac_address.is_some(),
        "MAC address should be present"
    );
    assert!(
        response_instance
            .mac_address
            .as_ref()
            .unwrap()
            .starts_with("BC:24:11"),
        "MAC address should use France Nuage OUI prefix"
    );
    assert_eq!(response_instance.vpc_id, Some(vpc.id.to_string()));
    assert_eq!(response_instance.vnet_id, Some(vnet.id.to_string()));
}
