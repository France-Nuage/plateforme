mod common;

use common::{
    Api, OnBehalfOf, seed_managed_service, seed_managed_service_instance,
    seed_managed_service_version,
};
use frn_rpc::v1::managed::GetInstanceRequest;
use tonic::{Code, Request};
use uuid::Uuid;

#[sqlx::test(migrations = "../migrations")]
async fn test_get_instance_returns_instance(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        None,
        "oci://registry.example.com/charts/vaultwarden",
    )
    .await;

    let instance = seed_managed_service_instance(&pool, service_id, version_id, "vaultwarden").await;

    let response = api
        .managed
        .services
        .get_instance(
            Request::new(GetInstanceRequest {
                instance_id: instance.id.to_string(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_ok());
    let resp = response.unwrap().into_inner().instance.unwrap();
    assert_eq!(resp.id, instance.id.to_string());
    assert_eq!(resp.status, "provisioning");

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_instance_returns_not_found(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .get_instance(
            Request::new(GetInstanceRequest {
                instance_id: Uuid::new_v4().to_string(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::NotFound);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_instance_returns_error_when_id_is_empty(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .get_instance(
            Request::new(GetInstanceRequest {
                instance_id: String::new(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);

    Ok(())
}
