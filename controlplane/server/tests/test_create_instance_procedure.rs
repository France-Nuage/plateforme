mod common;

use std::sync::{Arc, Mutex};

use common::{
    Api, OnBehalfOf, attach_test_deploy_label, seed_kubernetes_cluster, seed_managed_service,
    seed_managed_service_plan, seed_managed_service_version,
};
use fabrique::{Factory, Query};
use frn_core::identity::ServiceAccount;
use frn_core::managed::{
    CreateInstanceRequest as CoreCreateInstanceRequest, DeployManagedServiceParams, ManagedService,
    ManagedServices, PlatformConfig,
};
use frn_core::resourcemanager::{Organization, Project};
use frn_core::workflow::WorkflowScheduler;
use frn_rpc::v1::managed::CreateInstanceRequest;
use serde_json::json;
use spicedb::SpiceDB;
use sqlx::PgConnection;
use tonic::{Code, Request};
use uuid::Uuid;

/// Scheduler that records the cluster_id it was asked to deploy with, so a test
/// can assert the cluster resolved from the deploy_target is propagated to the
/// workflow.
#[derive(Clone)]
struct CapturingScheduler {
    captured: Arc<Mutex<Option<Uuid>>>,
}

impl WorkflowScheduler<DeployManagedServiceParams> for CapturingScheduler {
    async fn schedule(
        &self,
        _conn: &mut PgConnection,
        params: DeployManagedServiceParams,
    ) -> Result<(), String> {
        *self.captured.lock().unwrap() = Some(params.cluster_id);
        Ok(())
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_returns_instance_with_provisioning_status(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let organization = Organization::factory()
        .slug("acme".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await?;
    let cluster = seed_kubernetes_cluster(&pool, "prod-eu").await;
    attach_test_deploy_label(&pool, cluster.id).await;
    let project = Project::factory()
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await?;

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        Some("1.32.0"),
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;

    let response = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_slug: project.slug.clone(),
                organization_slug: organization.slug.clone(),
                service_slug: "vaultwarden".to_owned(),
                version_id: version_id.to_string(),
                plan_id: plan_id.to_string(),
                user_values: Some(json!({"domain": "vault.example.com"}).to_string()),
                secret_values: Some(json!({"smtp.password": "secret123"}).to_string()),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_ok());
    let instance = response.unwrap().into_inner().instance.unwrap();
    assert_eq!(instance.status, "provisioning");
    assert_eq!(instance.service_id, service_id.to_string());
    assert_eq!(instance.plan_id, Some(plan_id.to_string()));
    assert!(instance.namespace.starts_with("managed-acme-vaultwarden"));

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_fails_when_no_cluster_matches_deploy_target(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let organization = Organization::factory()
        .slug("acme".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await?;
    // The cluster exists and is healthy but does not carry the label required
    // by the service deploy_target.
    seed_kubernetes_cluster(&pool, "prod-eu").await;
    let project = Project::factory()
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await?;

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        Some("1.32.0"),
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;

    let status = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_slug: project.slug.clone(),
                organization_slug: organization.slug.clone(),
                service_slug: "vaultwarden".to_owned(),
                version_id: version_id.to_string(),
                plan_id: plan_id.to_string(),
                user_values: None,
                secret_values: None,
            })
            .on_behalf_of(&api.service_account),
        )
        .await
        .expect_err("must fail when no cluster matches the deploy_target");

    assert_eq!(status.code(), Code::FailedPrecondition);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_fails_when_service_has_no_deploy_target(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let organization = Organization::factory()
        .slug("acme".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await?;
    let cluster = seed_kubernetes_cluster(&pool, "prod-eu").await;
    attach_test_deploy_label(&pool, cluster.id).await;
    let project = Project::factory()
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await?;

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    // A service without deploy_target cannot be deployed, even when matching
    // clusters exist.
    ManagedService::query()
        .update()
        .set(ManagedService::DEPLOY_TARGET, None)
        .r#where(ManagedService::ID, "=", service_id)
        .execute(&pool)
        .await?;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        Some("1.32.0"),
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;

    let status = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_slug: project.slug.clone(),
                organization_slug: organization.slug.clone(),
                service_slug: "vaultwarden".to_owned(),
                version_id: version_id.to_string(),
                plan_id: plan_id.to_string(),
                user_values: None,
                secret_values: None,
            })
            .on_behalf_of(&api.service_account),
        )
        .await
        .expect_err("must fail when the service declares no deploy_target");

    assert_eq!(status.code(), Code::FailedPrecondition);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_propagates_matching_cluster_to_workflow(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization = Organization::factory()
        .slug("acme".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await?;
    let cluster = seed_kubernetes_cluster(&pool, "prod-eu").await;
    attach_test_deploy_label(&pool, cluster.id).await;
    let project = Project::factory()
        .organization_slug(organization.slug.clone())
        .create(&pool)
        .await?;

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        Some("1.32.0"),
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;

    let scheduler = CapturingScheduler {
        captured: Arc::new(Mutex::new(None)),
    };
    let mut managed = ManagedServices::new(
        SpiceDB::mock().await,
        pool.clone(),
        PlatformConfig {
            default_storage_class: None,
        },
    );
    let mut conn = pool.acquire().await?;

    let instance = managed
        .create_instance(
            &ServiceAccount::default(),
            &mut conn,
            &scheduler,
            CoreCreateInstanceRequest {
                project_slug: project.slug.clone(),
                organization_slug: organization.slug.clone(),
                service_slug: "vaultwarden".to_owned(),
                version_id,
                plan_id,
                user_values: None,
                secret_values: None,
            },
        )
        .await
        .expect("instance creation should succeed");

    assert_eq!(
        instance.cluster_id, cluster.id,
        "the instance must persist the cluster matching the deploy_target"
    );
    assert_eq!(
        *scheduler.captured.lock().unwrap(),
        Some(cluster.id),
        "the matching cluster must be propagated to the deploy workflow"
    );

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_returns_error_when_service_not_found(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_slug: "nonexistent-project".to_owned(),
                organization_slug: "nonexistent-org".to_owned(),
                service_slug: "nonexistent".to_owned(),
                version_id: Uuid::new_v4().to_string(),
                plan_id: Uuid::new_v4().to_string(),
                user_values: None,
                secret_values: None,
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::NotFound);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_returns_error_when_project_slug_is_empty(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_slug: String::new(),
                organization_slug: "some-org".to_owned(),
                service_slug: "vaultwarden".to_owned(),
                version_id: Uuid::new_v4().to_string(),
                plan_id: Uuid::new_v4().to_string(),
                user_values: None,
                secret_values: None,
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_returns_error_when_plan_id_is_empty(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_slug: "some-project".to_owned(),
                organization_slug: "some-org".to_owned(),
                service_slug: "vaultwarden".to_owned(),
                version_id: Uuid::new_v4().to_string(),
                plan_id: String::new(),
                user_values: None,
                secret_values: None,
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);

    Ok(())
}
