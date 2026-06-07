mod common;

use std::sync::{Arc, Mutex};

use common::{
    Api, OnBehalfOf, seed_kubernetes_cluster, seed_managed_service, seed_managed_service_plan,
    seed_managed_service_version,
};
use fabrique::Factory;
use frn_core::identity::ServiceAccount;
use frn_core::managed::{
    CreateInstanceRequest as CoreCreateInstanceRequest, DeployManagedServiceParams,
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
/// can assert the project's cluster is propagated to the workflow.
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
        .parent_id(None)
        .create(&pool)
        .await?;
    let cluster = seed_kubernetes_cluster(&pool, "prod-eu").await;
    let project = Project::factory()
        .organization_id(organization.id)
        .cluster_id(Some(cluster.id))
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
                project_id: project.id.to_string(),
                organization_id: organization.id.to_string(),
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
async fn test_create_instance_fails_when_project_has_no_cluster(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let organization = Organization::factory()
        .slug("acme".to_owned())
        .parent_id(None)
        .create(&pool)
        .await?;
    let project = Project::factory()
        .organization_id(organization.id)
        .cluster_id(None)
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
                project_id: project.id.to_string(),
                organization_id: organization.id.to_string(),
                service_slug: "vaultwarden".to_owned(),
                version_id: version_id.to_string(),
                plan_id: plan_id.to_string(),
                user_values: None,
                secret_values: None,
            })
            .on_behalf_of(&api.service_account),
        )
        .await
        .expect_err("must fail when the project has no cluster assigned");

    assert_eq!(status.code(), Code::FailedPrecondition);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_propagates_project_cluster_to_workflow(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization = Organization::factory()
        .slug("acme".to_owned())
        .parent_id(None)
        .create(&pool)
        .await?;
    let cluster = seed_kubernetes_cluster(&pool, "prod-eu").await;
    let project = Project::factory()
        .organization_id(organization.id)
        .cluster_id(Some(cluster.id))
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

    managed
        .create_instance(
            &ServiceAccount::default(),
            &mut conn,
            &scheduler,
            CoreCreateInstanceRequest {
                project_id: project.id,
                organization_id: organization.id,
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
        *scheduler.captured.lock().unwrap(),
        Some(cluster.id),
        "the project's cluster must be propagated to the deploy workflow"
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
                project_id: Uuid::new_v4().to_string(),
                organization_id: Uuid::new_v4().to_string(),
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
async fn test_create_instance_returns_error_when_project_id_is_empty(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_id: String::new(),
                organization_id: Uuid::new_v4().to_string(),
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
                project_id: Uuid::new_v4().to_string(),
                organization_id: Uuid::new_v4().to_string(),
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
