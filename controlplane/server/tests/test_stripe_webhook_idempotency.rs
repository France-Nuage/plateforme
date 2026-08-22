//! Black-box tests for Stripe webhook idempotency.
//!
//! Drives `Billing::dispatch_webhook_event` against a real database (only the
//! Stripe API boundary is stubbed) to verify that a duplicated webhook delivery
//! is processed exactly once.

use std::sync::Arc;

use fabrique::Factory;
use frn_core::billing::{
    Billing, BillingError, CheckoutMetadata, CheckoutSessionResult, StripeClient,
    StripeWebhookEvent,
};
use frn_core::managed::{DeployManagedServiceParams, ManagedServices, PlatformConfig};
use frn_core::resourcemanager::Organization;
use frn_core::workflow::WorkflowScheduler;
use frn_crypto::Kek;
use spicedb::SpiceDB;
use sqlx::{PgConnection, Pool, Postgres};
use uuid::Uuid;

use crate::common::{seed_managed_service, seed_managed_service_plan};

mod common;

/// Stripe client stub: the paths exercised here never call out to Stripe, so any
/// invocation is a test failure.
#[derive(Clone)]
struct UnreachableStripeClient;

impl StripeClient for UnreachableStripeClient {
    async fn create_customer(&self, _: &str, _: &str) -> Result<String, BillingError> {
        unreachable!("StripeClient::create_customer must not be called in this test")
    }

    async fn create_checkout_session(
        &self,
        _: &str,
        _: &str,
        _: CheckoutMetadata,
        _: &str,
        _: &str,
    ) -> Result<CheckoutSessionResult, BillingError> {
        unreachable!("StripeClient::create_checkout_session must not be called in this test")
    }

    async fn cancel_subscription(&self, _: &str) -> Result<(), BillingError> {
        unreachable!("StripeClient::cancel_subscription must not be called in this test")
    }

    async fn delete_customer(&self, _: &str) -> Result<(), BillingError> {
        unreachable!("StripeClient::delete_customer must not be called in this test")
    }

    async fn ensure_product(
        &self,
        _: &str,
        _: &str,
        _: Option<&str>,
    ) -> Result<String, BillingError> {
        unreachable!("StripeClient::ensure_product must not be called in this test")
    }

    async fn ensure_price(
        &self,
        _: &frn_core::billing::PriceSpec,
    ) -> Result<frn_core::billing::EnsurePriceResult, BillingError> {
        unreachable!("StripeClient::ensure_price must not be called in this test")
    }

    async fn delete_or_archive_price(&self, _: &str) -> Result<(), BillingError> {
        unreachable!("StripeClient::delete_or_archive_price must not be called in this test")
    }

    async fn archive_product(&self, _: &str) -> Result<(), BillingError> {
        unreachable!("StripeClient::archive_product must not be called in this test")
    }

    async fn list_managed_prices(
        &self,
    ) -> Result<Vec<frn_core::billing::ManagedPrice>, BillingError> {
        unreachable!("StripeClient::list_managed_prices must not be called in this test")
    }

    async fn list_managed_products(
        &self,
    ) -> Result<Vec<frn_core::billing::ManagedProduct>, BillingError> {
        unreachable!("StripeClient::list_managed_products must not be called in this test")
    }
}

/// Scheduler stub: the status-change path schedules no workflow, but the dispatch
/// signature still requires a scheduler.
#[derive(Clone)]
struct NoopWorkflowScheduler;

impl WorkflowScheduler<DeployManagedServiceParams> for NoopWorkflowScheduler {
    async fn schedule(
        &self,
        _conn: &mut PgConnection,
        _params: DeployManagedServiceParams,
    ) -> Result<(), String> {
        Ok(())
    }
}

fn build_billing(
    pool: &Pool<Postgres>,
    auth: SpiceDB,
) -> Billing<SpiceDB, UnreachableStripeClient> {
    let managed = ManagedServices::new(
        auth,
        pool.clone(),
        PlatformConfig {
            default_storage_class: None,
            cnpg_backup_enabled: false,
            deployment_labels: std::collections::BTreeMap::new(),
            deployment_annotations: std::collections::BTreeMap::new(),
        },
    );
    Billing::new(
        pool.clone(),
        UnreachableStripeClient,
        managed,
        Arc::new(Kek::from_bytes([7u8; 32])),
        "https://console.test/success".to_owned(),
        "https://console.test/cancel".to_owned(),
    )
}

/// Seeds an active subscription (with its customer) bound to `stripe_subscription_id`.
async fn seed_active_subscription(
    pool: &Pool<Postgres>,
    organization_slug: &str,
    plan_id: Uuid,
    stripe_subscription_id: &str,
) -> Uuid {
    let customer_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO billing.customer (id, organization_slug, stripe_customer_id)
           VALUES (gen_random_uuid(), $1, $2)
           RETURNING id"#,
    )
    .bind(organization_slug)
    .bind(format!("cus_{stripe_subscription_id}"))
    .fetch_one(pool)
    .await
    .expect("could not seed billing customer");

    sqlx::query_scalar(
        r#"INSERT INTO billing.subscription
               (id, customer_id, stripe_subscription_id, plan_id, status, billing_period)
           VALUES (gen_random_uuid(), $1, $2, $3, 'active', 'monthly')
           RETURNING id"#,
    )
    .bind(customer_id)
    .bind(stripe_subscription_id)
    .bind(plan_id)
    .fetch_one(pool)
    .await
    .expect("could not seed billing subscription")
}

/// A second, identical webhook delivery must be a no-op: the subscription is
/// updated once and the event is recorded once. Without idempotency the retry
/// would re-run the handler and fail on the invalid `past_due -> past_due`
/// transition.
#[sqlx::test(migrations = "../migrations")]
async fn test_duplicate_webhook_delivery_is_processed_once(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization = Organization::factory()
        .slug("wh-org".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not seed organization");
    let service_id = seed_managed_service(&pool, "wh-postgres", "WH Postgres", "database").await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "wh-postgres-standard", "Standard").await;
    let subscription_id =
        seed_active_subscription(&pool, &organization.slug, plan_id, "sub_wh_test").await;

    let billing = build_billing(&pool, SpiceDB::mock().await);
    let scheduler = NoopWorkflowScheduler;
    let mut conn = pool.acquire().await?;

    let event = StripeWebhookEvent {
        event_id: "evt_wh_duplicate".to_owned(),
        event_type: "invoice.payment_failed".to_owned(),
        checkout_session_id: None,
        stripe_subscription_id: Some("sub_wh_test".to_owned()),
        period_start: None,
        period_end: None,
    };

    billing
        .dispatch_webhook_event(&mut conn, &scheduler, event.clone())
        .await
        .expect("first delivery must succeed");

    billing
        .dispatch_webhook_event(&mut conn, &scheduler, event.clone())
        .await
        .expect("duplicate delivery must be a no-op");

    let status: String =
        sqlx::query_scalar("SELECT status FROM billing.subscription WHERE id = $1")
            .bind(subscription_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        status, "past_due",
        "subscription must be processed exactly once"
    );

    let processed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM billing.processed_event WHERE event_id = $1")
            .bind("evt_wh_duplicate")
            .fetch_one(&pool)
            .await?;
    assert_eq!(processed, 1, "event must be recorded exactly once");

    Ok(())
}

/// A distinct event on the same subscription is still processed: the idempotency
/// claim keys on the event id, not the subscription.
#[sqlx::test(migrations = "../migrations")]
async fn test_distinct_events_are_each_processed(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization = Organization::factory()
        .slug("wh-org-two".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not seed organization");
    let service_id = seed_managed_service(&pool, "wh-redis", "WH Redis", "database").await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "wh-redis-standard", "Standard").await;
    seed_active_subscription(&pool, &organization.slug, plan_id, "sub_wh_test_2").await;

    let billing = build_billing(&pool, SpiceDB::mock().await);
    let scheduler = NoopWorkflowScheduler;
    let mut conn = pool.acquire().await?;

    let base = StripeWebhookEvent {
        event_id: String::new(),
        event_type: "invoice.payment_failed".to_owned(),
        checkout_session_id: None,
        stripe_subscription_id: Some("sub_wh_test_2".to_owned()),
        period_start: None,
        period_end: None,
    };

    // past_due, then a second distinct event flips it back to active.
    billing
        .dispatch_webhook_event(
            &mut conn,
            &scheduler,
            StripeWebhookEvent {
                event_id: "evt_first".to_owned(),
                ..base.clone()
            },
        )
        .await
        .expect("first event must succeed");
    billing
        .dispatch_webhook_event(
            &mut conn,
            &scheduler,
            StripeWebhookEvent {
                event_id: "evt_second".to_owned(),
                event_type: "invoice.paid".to_owned(),
                period_start: Some(1_700_000_000),
                period_end: Some(1_702_600_000),
                ..base.clone()
            },
        )
        .await
        .expect("second distinct event must succeed");

    let processed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM billing.processed_event")
        .fetch_one(&pool)
        .await?;
    assert_eq!(processed, 2, "each distinct event must be recorded");

    Ok(())
}

/// An event whose subscription is unknown to this instance must be acked as a
/// no-op, not fail. Stripe broadcasts every event to all listeners on the same
/// account, so a control plane routinely receives events for subscriptions owned
/// by another environment sharing the sandbox (concurrent ephemeral CI runs). A
/// 500 there would make Stripe retry forever and wedge the delivery; instead the
/// handler returns Ok so Stripe marks it delivered, and no local row is touched.
#[sqlx::test(migrations = "../migrations")]
async fn test_event_for_unknown_subscription_is_ignored(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization = Organization::factory()
        .slug("wh-org-foreign".to_owned())
        .parent_slug(None)
        .create(&pool)
        .await
        .expect("could not seed organization");
    let service_id = seed_managed_service(&pool, "wh-mysql", "WH MySQL", "database").await;
    let plan_id =
        seed_managed_service_plan(&pool, service_id, "wh-mysql-standard", "Standard").await;
    // This instance owns exactly one subscription...
    let owned_id =
        seed_active_subscription(&pool, &organization.slug, plan_id, "sub_wh_owned").await;

    let billing = build_billing(&pool, SpiceDB::mock().await);
    let scheduler = NoopWorkflowScheduler;
    let mut conn = pool.acquire().await?;

    // ...but the event references a subscription created by another environment.
    let foreign_event = StripeWebhookEvent {
        event_id: "evt_foreign".to_owned(),
        event_type: "customer.subscription.deleted".to_owned(),
        checkout_session_id: None,
        stripe_subscription_id: Some("sub_belongs_to_another_env".to_owned()),
        period_start: None,
        period_end: None,
    };

    billing
        .dispatch_webhook_event(&mut conn, &scheduler, foreign_event)
        .await
        .expect("an event for an unknown subscription must be a no-op, not an error");

    // The locally owned subscription is left untouched.
    let owned_status: String =
        sqlx::query_scalar("SELECT status FROM billing.subscription WHERE id = $1")
            .bind(owned_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        owned_status, "active",
        "a foreign event must not alter this instance's subscriptions"
    );

    Ok(())
}
