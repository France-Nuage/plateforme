mod common;

use common::{Api, OnBehalfOf, seed_managed_service, seed_managed_service_version};
use frn_rpc::v1::managed::CreateInstanceRequest;
use serde_json::json;
use tonic::{Code, Request};
use uuid::Uuid;

#[sqlx::test(migrations = "../migrations")]
async fn test_create_instance_returns_instance_with_provisioning_status(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        Some("1.32.0"),
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;

    let response = api
        .managed
        .services
        .create_instance(
            Request::new(CreateInstanceRequest {
                project_id: Uuid::new_v4().to_string(),
                organization_id: Uuid::new_v4().to_string(),
                service_slug: "vaultwarden".to_owned(),
                version_id: version_id.to_string(),
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
    assert!(instance.namespace.starts_with("managed-vaultwarden-"));

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
