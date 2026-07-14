use crate::common::{Api, OnBehalfOf};
use fabrique::{Factory, Query};
use frn_core::compute::{Hypervisor, Instance, Zone};
use frn_core::resourcemanager::{Organization, Project};
use frn_rpc::v1::compute::UpdateInstanceRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_the_update_instance_procedure_can_change_project(pool: sqlx::PgPool) {
    // Arrange a test api and the required data
    let mut api = Api::start(&pool).await.expect("could not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .slug("test-org".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not create organization");

    // Create project A explicitly
    let project_a = Project::factory()
        .slug("project-a".to_owned())
        .name("project-a".into())
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await
        .expect("could not create project A");

    // Create project B explicitly
    let project_b = Project::factory()
        .slug("project-b".to_owned())
        .name("project-b".into())
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await
        .expect("could not create project B");

    // Create an instance in project A
    let instance = Instance::factory()
        .for_hypervisor(
            Hypervisor::factory()
                .for_zone(Zone::factory())
                .organization_slug(organization.slug.clone())
                .url(mock_url),
        )
        .project_slug(project_a.slug.clone())
        .name("test-instance".into())
        .distant_id("100".into())
        .zero_trust_network_id(None)
        .create(&pool)
        .await
        .expect("could not create instance");

    let original_project_slug = instance.project_slug.clone();

    // Act: Update the instance to move it to project B
    let request = Request::new(UpdateInstanceRequest {
        id: instance.id.to_string(),
        name: None,
        project_slug: Some(project_b.slug.clone()),
    })
    .on_behalf_of(&api.service_account);

    let result = api.compute.instances.update(request).await;

    // Assert the result
    assert!(result.is_ok(), "Update should succeed: {:?}", result.err());

    let response = result.unwrap().into_inner();
    let updated_instance = response.instance.expect("Response should contain instance");

    assert_eq!(updated_instance.id, instance.id.to_string());
    assert_eq!(updated_instance.project_slug, project_b.slug.clone());
    assert_ne!(updated_instance.project_slug, original_project_slug);
    assert_eq!(updated_instance.name, "test-instance");

    // Verify in database
    let db_instance = Instance::find(&pool, instance.id)
        .await
        .expect("could not find instance");
    assert_eq!(db_instance.project_slug, project_b.slug);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_the_update_instance_procedure_can_change_name(pool: sqlx::PgPool) {
    // Arrange a test api and the required data
    let mut api = Api::start(&pool).await.expect("could not start api");
    let mock_url = api.mock_server.url();

    let organization = Organization::factory()
        .slug("test-org".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not create organization");

    // Create a project and an instance
    let project = Project::factory()
        .slug("test-project".to_owned())
        .organization_slug(organization.slug.clone())
        .name("test-project".into())
        .create(&pool)
        .await
        .expect("could not create project");

    let instance = Instance::factory()
        .for_hypervisor(
            Hypervisor::factory()
                .for_zone(Zone::factory())
                .organization_slug(organization.slug.clone())
                .url(mock_url),
        )
        .project_slug(project.slug)
        .name("old-name".into())
        .distant_id("101".into())
        .zero_trust_network_id(None)
        .create(&pool)
        .await
        .expect("could not create instance");

    let project_slug = instance.project_slug.clone();

    // Act: Update the instance name
    let request = Request::new(UpdateInstanceRequest {
        id: instance.id.to_string(),
        name: Some("new-name".to_string()),
        project_slug: None,
    })
    .on_behalf_of(&api.service_account);

    let result = api.compute.instances.update(request).await;

    // Assert the result
    assert!(result.is_ok(), "Update should succeed: {:?}", result.err());

    let response = result.unwrap().into_inner();
    let updated_instance = response.instance.expect("Response should contain instance");

    assert_eq!(updated_instance.name, "new-name");
    assert_eq!(updated_instance.project_slug, project_slug);

    // Verify in database
    let db_instance = Instance::find(&pool, instance.id)
        .await
        .expect("could not find instance");
    assert_eq!(db_instance.name, "new-name");
}
