use crate::common::{Api, OnBehalfOf};
use frn_core::compute::Instance;
use frn_core::resourcemanager::{Organization, Project};
use frn_rpc::v1::compute::UpdateInstanceRequest;
use sqlx::types::Uuid;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_update_instance_procedure_can_change_project(pool: sqlx::PgPool) {
    // Arrange a test api and the required data
    let mut api = Api::start(&pool).await.expect("could not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .expect("could not create organization");

    // Create project A explicitly
    let project_a = Project::factory()
        .id(Uuid::new_v4())
        .name("project-a".into())
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create project A");

    // Create project B explicitly
    let project_b = Project::factory()
        .id(Uuid::new_v4())
        .name("project-b".into())
        .organization_id(organization.id)
        .create(&pool)
        .await
        .expect("could not create project B");

    // Create an instance in project A
    let instance = Instance::factory()
        .for_hypervisor(move |hypervisor| {
            hypervisor
                .for_zone(|zone| zone)
                .organization_id(organization.id)
                .url(mock_url)
        })
        .project_id(project_a.id)
        .name("test-instance".into())
        .distant_id("100".into())
        .create(&pool)
        .await
        .expect("could not create instance");

    let original_project_id = instance.project_id;

    // Act: Update the instance to move it to project B
    let request = Request::new(UpdateInstanceRequest {
        id: instance.id.to_string(),
        name: None,
        project_id: Some(project_b.id.to_string()),
    })
    .on_behalf_of(&api.service_account);

    let result = api.compute.instances.update(request).await;

    // Assert the result
    assert!(result.is_ok(), "Update should succeed: {:?}", result.err());

    let response = result.unwrap().into_inner();
    let updated_instance = response.instance.expect("Response should contain instance");

    assert_eq!(updated_instance.id, instance.id.to_string());
    assert_eq!(updated_instance.project_id, project_b.id.to_string());
    assert_ne!(updated_instance.project_id, original_project_id.to_string());
    assert_eq!(updated_instance.name, "test-instance");

    // Verify in database
    let db_instance = Instance::find_one_by_id(&pool, instance.id)
        .await
        .expect("could not find instance");
    assert_eq!(db_instance.project_id, project_b.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_update_instance_procedure_can_change_name(pool: sqlx::PgPool) {
    // Arrange a test api and the required data
    let mut api = Api::start(&pool).await.expect("could not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .id(Uuid::new_v4())
        .create(&pool)
        .await
        .expect("could not create organization");

    // Create an instance
    let instance = Instance::factory()
        .for_hypervisor(move |hypervisor| {
            hypervisor
                .for_zone(|zone| zone)
                .organization_id(organization.id)
                .url(mock_url)
        })
        .for_project(move |project| project.organization_id(organization.id))
        .name("old-name".into())
        .distant_id("101".into())
        .create(&pool)
        .await
        .expect("could not create instance");

    let project_id = instance.project_id;

    // Act: Update the instance name
    let request = Request::new(UpdateInstanceRequest {
        id: instance.id.to_string(),
        name: Some("new-name".to_string()),
        project_id: None,
    })
    .on_behalf_of(&api.service_account);

    let result = api.compute.instances.update(request).await;

    // Assert the result
    assert!(result.is_ok(), "Update should succeed: {:?}", result.err());

    let response = result.unwrap().into_inner();
    let updated_instance = response.instance.expect("Response should contain instance");

    assert_eq!(updated_instance.name, "new-name");
    assert_eq!(updated_instance.project_id, project_id.to_string());

    // Verify in database
    let db_instance = Instance::find_one_by_id(&pool, instance.id)
        .await
        .expect("could not find instance");
    assert_eq!(db_instance.name, "new-name");
}
