mod common;

use common::{
    Api, OnBehalfOf, seed_managed_service, seed_managed_service_instance,
    seed_managed_service_version,
};
use frn_rpc::v1::managed::ListInstancesRequest;
use tonic::{Code, Request};
use uuid::Uuid;

#[sqlx::test(migrations = "../migrations")]
async fn test_list_instances_returns_instances_for_project(
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

    let instance = seed_managed_service_instance(&pool, version_id, "vaultwarden").await;
    let project_id = instance.project_id;

    let response = api
        .managed
        .services
        .list_instances(
            Request::new(ListInstancesRequest {
                project_id: project_id.to_string(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_ok());
    let instances = response.unwrap().into_inner().instances;
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].status, "provisioning");

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_instances_returns_empty_for_unknown_project(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .list_instances(
            Request::new(ListInstancesRequest {
                project_id: Uuid::new_v4().to_string(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_ok());
    let instances = response.unwrap().into_inner().instances;
    assert!(instances.is_empty());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_instances_returns_error_when_project_id_is_empty(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .list_instances(
            Request::new(ListInstancesRequest {
                project_id: String::new(),
            })
            .on_behalf_of(&api.service_account),
        )
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);

    Ok(())
}
