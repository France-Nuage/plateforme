mod common;

use common::{IntoCi, seed_managed_service};
use frn_rpc::v1::managed::{
    ListPlansRequest, ManagedServicePlanEntitlementProto, SyncPlanEntry, SyncPlansRequest,
};
use sqlx::PgPool;
use tonic::{Code, Request};

#[sqlx::test(migrations = "../migrations")]
async fn test_sync_plans_creates_plans(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let response = api
        .managed
        .services
        .sync_plans(
            Request::new(SyncPlansRequest {
                service_slug: "vaultwarden".to_owned(),
                plans: vec![SyncPlanEntry {
                    slug: "vaultwarden-standard".to_owned(),
                    name: "Standard".to_owned(),
                    description: Some("Instance standard".to_owned()),
                    status: "active".to_owned(),
                    highlighted: false,
                    values_override: None,
                    entitlements: vec![ManagedServicePlanEntitlementProto {
                        key: "support_level".to_owned(),
                        label: "Support".to_owned(),
                        value: "Email".to_owned(),
                    }],
                    price_monthly_cents: Some(999),
                    price_yearly_cents: Some(10789),
                }],
            })
            .into_ci(),
        )
        .await;

    assert!(response.is_ok());
    let plans = response.unwrap().into_inner().plans;
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].slug, "vaultwarden-standard");
    assert_eq!(plans[0].name, "Standard");
    assert_eq!(plans[0].price_monthly_cents, Some(999));

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_sync_plans_upserts_existing_plan(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let entry = SyncPlanEntry {
        slug: "vaultwarden-standard".to_owned(),
        name: "Standard".to_owned(),
        description: None,
        status: "active".to_owned(),
        highlighted: false,
        values_override: None,
        entitlements: vec![],
        price_monthly_cents: Some(999),
        price_yearly_cents: Some(10789),
    };

    api.managed
        .services
        .sync_plans(
            Request::new(SyncPlansRequest {
                service_slug: "vaultwarden".to_owned(),
                plans: vec![entry.clone()],
            })
            .into_ci(),
        )
        .await
        .expect("first sync should succeed");

    let updated_entry = SyncPlanEntry {
        name: "Standard v2".to_owned(),
        price_monthly_cents: Some(1499),
        ..entry
    };

    api.managed
        .services
        .sync_plans(
            Request::new(SyncPlansRequest {
                service_slug: "vaultwarden".to_owned(),
                plans: vec![updated_entry],
            })
            .into_ci(),
        )
        .await
        .expect("second sync should succeed");

    let list_response = api
        .managed
        .services
        .list_plans(Request::new(ListPlansRequest {
            service_slug: "vaultwarden".to_owned(),
        }))
        .await
        .expect("list should succeed");

    let plans = list_response.into_inner().plans;
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].name, "Standard v2");
    assert_eq!(plans[0].price_monthly_cents, Some(1499));

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_sync_plans_rejects_unauthenticated(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let response = api
        .managed
        .services
        .sync_plans(Request::new(SyncPlansRequest {
            service_slug: "vaultwarden".to_owned(),
            plans: vec![],
        }))
        .await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn test_sync_plans_rejects_wrong_token(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    use tonic::metadata::MetadataValue;

    let mut api = common::Api::start(&pool)
        .await
        .expect("could not start api");

    seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;

    let mut request = Request::new(SyncPlansRequest {
        service_slug: "vaultwarden".to_owned(),
        plans: vec![],
    });
    let bad_token =
        MetadataValue::try_from("Bearer wrong-token").expect("could not create metadata value");
    request.metadata_mut().insert("authorization", bad_token);

    let response = api.managed.services.sync_plans(request).await;

    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);

    Ok(())
}
