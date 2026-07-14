mod common;

use common::{seed_managed_service, seed_managed_service_plan};
use frn_rpc::v1::managed::ListPlansRequest;
use serde_json::json;
use sqlx::PgPool;
use tonic::{Code, Request};

#[sqlx::test(migrations = "../migrations")]
async fn test_list_plans_returns_empty_when_no_plans(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await;

    let plans = response
        .expect("list_plans must succeed")
        .into_inner()
        .plans;
    assert!(plans.is_empty());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_plans_returns_active_plans(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;
    seed_managed_service_plan(&pool, service_id, "vaultwarden-enterprise", "Enterprise").await;

    let response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await;

    let plans = response
        .expect("list_plans must succeed")
        .into_inner()
        .plans;
    assert_eq!(plans.len(), 2);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_plans_excludes_archived(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;

    sqlx::query(
        r#"INSERT INTO managed.service_plan
               (id, service_id, slug, name, status, entitlements)
           VALUES (gen_random_uuid(), $1, 'vaultwarden-legacy', 'Legacy', 'archived', $2)"#,
    )
    .bind(service_id)
    .bind(json!([]))
    .execute(&pool)
    .await?;

    let response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await;

    let plans = response
        .expect("list_plans must succeed")
        .into_inner()
        .plans;
    assert_eq!(plans.len(), 1);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_plans_returns_not_found_for_unknown_service(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    let response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: "nonexistent".to_owned(),
        }))
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::NotFound);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_plans_returns_error_when_slug_is_empty(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    let response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: String::new(),
        }))
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_plans_returns_entitlements_and_prices(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    seed_managed_service_plan(&pool, service_id, "vaultwarden-standard", "Standard").await;

    let response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await;

    let plan = &response
        .expect("list_plans must succeed")
        .into_inner()
        .plans[0];
    assert_eq!(plan.price_monthly_cents, Some(999));

    Ok(())
}
