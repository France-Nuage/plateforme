use crate::common::{Api, seed_managed_service};
use frn_rpc::v1::managed::GetServiceRequest;
use tonic::{Code, Request};

mod common;

#[sqlx::test(migrations = "../migrations")]
async fn test_get_managed_service_returns_service(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let response = api
        .managed
        .services
        .get_service(Request::new(GetServiceRequest {
            slug: "vaultwarden".to_owned(),
        }))
        .await;

    let service = response
        .expect("get_service must succeed")
        .into_inner()
        .service
        .expect("response must contain a service");
    assert_eq!(service.slug, "vaultwarden");

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_managed_service_returns_not_found_for_unknown_slug(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .get_service(Request::new(GetServiceRequest {
            slug: "nonexistent".to_owned(),
        }))
        .await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_managed_service_rejects_empty_slug(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .managed
        .services
        .get_service(Request::new(GetServiceRequest {
            slug: String::new(),
        }))
        .await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);

    Ok(())
}
