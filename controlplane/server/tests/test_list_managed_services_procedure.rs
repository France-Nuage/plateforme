use crate::common::{
    Api, clear_managed_services, deactivate_managed_service, seed_managed_service,
};
use frn_rpc::v1::managed::ListServicesRequest;
use tonic::Request;

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_list_managed_services_returns_empty_when_no_services(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    clear_managed_services(&pool).await;
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .list_services(Request::new(ListServicesRequest {}))
        .await;

    let resp = response.expect("list_services must succeed").into_inner();
    assert!(resp.services.is_empty());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_managed_services_returns_active_services(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    clear_managed_services(&pool).await;
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    seed_managed_service(&pool, "nextcloud", "Nextcloud", "collaboration").await;

    let response = api
        .managed
        .services
        .list_services(Request::new(ListServicesRequest {}))
        .await;

    let resp = response.expect("list_services must succeed").into_inner();
    assert_eq!(resp.services.len(), 2);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_managed_services_excludes_deactivated(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    clear_managed_services(&pool).await;
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let deactivated_id =
        seed_managed_service(&pool, "old-service", "Old Service", "analytics").await;

    deactivate_managed_service(&pool, deactivated_id).await;

    let response = api
        .managed
        .services
        .list_services(Request::new(ListServicesRequest {}))
        .await;

    let resp = response.expect("list_services must succeed").into_inner();
    assert_eq!(resp.services.len(), 1);

    Ok(())
}
