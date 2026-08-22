//! Billing and subscription management for managed services.
//!
//! Provides entity definitions, Stripe integration traits, and service layers
//! for handling checkout sessions, subscriptions, and webhook processing.

mod catalog;
mod checkout;
mod customer;
pub mod stripe;
mod subscription;
mod webhook;

pub use catalog::*;
pub use checkout::*;
pub use webhook::*;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use fabrique::Model;
use frn_crypto::Kek;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use thiserror::Error;
use uuid::Uuid;

use crate::authorization::Authorize;
use crate::managed::{ManagedServiceError, ManagedServices};

#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "billing.customer")]
pub struct BillingCustomer {
    #[fabrique(primary_key)]
    pub id: Uuid,
    pub organization_slug: String,
    pub stripe_customer_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    PendingPayment,
    Active,
    PastDue,
    Canceled,
    Incomplete,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionStatus::PendingPayment => write!(f, "pending_payment"),
            SubscriptionStatus::Active => write!(f, "active"),
            SubscriptionStatus::PastDue => write!(f, "past_due"),
            SubscriptionStatus::Canceled => write!(f, "canceled"),
            SubscriptionStatus::Incomplete => write!(f, "incomplete"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BillingPeriod {
    Monthly,
    Yearly,
}

impl std::fmt::Display for BillingPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BillingPeriod::Monthly => write!(f, "monthly"),
            BillingPeriod::Yearly => write!(f, "yearly"),
        }
    }
}

#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "billing.subscription")]
pub struct BillingSubscription {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(belongs_to = BillingCustomer)]
    pub customer_id: Uuid,
    pub stripe_subscription_id: Option<String>,
    pub stripe_checkout_session_id: Option<String>,
    pub plan_id: Uuid,
    pub instance_id: Option<Uuid>,
    pub status: SubscriptionStatus,
    pub billing_period: BillingPeriod,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "billing.processed_event")]
pub struct ProcessedEvent {
    #[fabrique(primary_key)]
    pub event_id: String,
    pub event_type: String,
    pub processed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "billing.pending_instance_params")]
pub struct PendingInstanceParams {
    #[fabrique(primary_key)]
    pub subscription_id: Uuid,
    pub service_slug: String,
    pub version_id: Uuid,
    pub project_slug: String,
    pub organization_slug: String,
    pub user_values: Option<serde_json::Value>,
    pub secret_ciphertext: Option<Vec<u8>>,
    pub secret_nonce: Option<Vec<u8>>,
    pub secret_dek_ciphertext: Option<Vec<u8>>,
    pub secret_dek_nonce: Option<Vec<u8>>,
    pub secret_key_version: Option<i32>,
    pub secret_algorithm: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("fabrique error: {0}")]
    Fabrique(#[from] fabrique::Error),
    #[error("stripe error: {0}")]
    Stripe(String),
    #[error("customer not found for organization: {0}")]
    CustomerNotFound(String),
    #[error("subscription not found: {0}")]
    SubscriptionNotFound(String),
    #[error("invalid subscription status transition: {from} -> {to}")]
    InvalidStatusTransition {
        from: SubscriptionStatus,
        to: SubscriptionStatus,
    },
    #[error("plan requires no payment: {0}")]
    PlanRequiresNoPayment(String),
    #[error("plan has no stripe price for period {period}: {plan_slug}")]
    MissingStripePrice {
        plan_slug: String,
        period: BillingPeriod,
    },
    #[error("webhook signature verification failed")]
    InvalidWebhookSignature,
    #[error("duplicate event: {0}")]
    DuplicateEvent(String),
    #[error("pending instance params not found for subscription: {0}")]
    PendingParamsNotFound(Uuid),
    #[error("managed service error: {0}")]
    ManagedService(#[from] ManagedServiceError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("encryption error: {0}")]
    Encryption(#[from] frn_crypto::EncryptionError),
    #[error("invalid unix timestamp: {0}")]
    InvalidTimestamp(i64),
}

/// Recurring billing interval for a Stripe price.
///
/// Maps a [`BillingPeriod`] onto the Stripe recurring interval used when
/// creating prices during catalogue reconciliation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceInterval {
    Month,
    Year,
}

impl From<BillingPeriod> for PriceInterval {
    fn from(period: BillingPeriod) -> Self {
        match period {
            BillingPeriod::Monthly => PriceInterval::Month,
            BillingPeriod::Yearly => PriceInterval::Year,
        }
    }
}

/// Stripe `metadata` key marking objects owned by the France Nuage catalogue.
///
/// The reconciler tags every product and price it creates with
/// `managed_by = france-nuage-catalog`. Pruning only ever touches objects
/// carrying this tag, so unrelated Stripe data is never affected.
pub const CATALOG_MANAGED_BY_KEY: &str = "managed_by";

/// Value of [`CATALOG_MANAGED_BY_KEY`] for catalogue-owned objects.
pub const CATALOG_MANAGED_BY_VALUE: &str = "france-nuage-catalog";

/// Declarative specification of a recurring price to reconcile into Stripe.
///
/// This is the input to [`StripeClient::ensure_price`]. It carries the desired
/// state (amount, currency, interval, nickname) plus the stable `lookup_key`
/// that lets the reconciler find and converge the price idempotently,
/// independently of Stripe's opaque generated `price_...` id.
#[derive(Debug, Clone)]
pub struct PriceSpec {
    /// Stable Stripe lookup key, declared in the catalogue.
    pub lookup_key: String,
    /// Stripe product id the price belongs to.
    pub product_id: String,
    /// Amount in the currency's smallest unit (e.g. cents).
    pub unit_amount_cents: i64,
    /// Three-letter ISO currency code, lowercase (e.g. `eur`).
    pub currency: String,
    /// Recurring billing interval, or `None` for a one-time price.
    pub interval: Option<PriceInterval>,
    /// Optional Stripe nickname (internal label, hidden from customers). A
    /// mutable field: changing it updates the existing price without recreating.
    pub nickname: Option<String>,
}

/// Result of reconciling a single price into Stripe.
///
/// Reports the resulting active `price_...` id and whether a new price was
/// created (amount changed / first creation) versus an existing price reused.
#[derive(Debug, Clone)]
pub struct EnsurePriceResult {
    /// The Stripe id of the active price after reconciliation.
    pub price_id: String,
    /// Whether a new price object was created during this call.
    pub created: bool,
}

/// A catalogue-owned Stripe price, as seen when listing the prune perimeter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedPrice {
    /// Stripe price id (`price_...`).
    pub id: String,
    /// The price's lookup key, if any.
    pub lookup_key: Option<String>,
}

/// A catalogue-owned Stripe product, as seen when listing the prune perimeter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedProduct {
    /// Stripe product id (`prod_...`).
    pub id: String,
}

/// Stripe API abstraction for testability.
///
/// Beyond the transactional operations (customer/checkout/subscription), this
/// trait exposes catalogue reconciliation primitives (`ensure_product`,
/// `ensure_price`, `archive_price`, `archive_product`) used to push the
/// declarative service catalogue into Stripe. All reconciliation methods are
/// idempotent and safe to run concurrently: products are keyed by a stable id
/// and prices by a stable `lookup_key`.
#[trait_variant::make(Send)]
pub trait StripeClient: Clone + Send + Sync {
    async fn create_customer(
        &self,
        organization_slug: &str,
        organization_name: &str,
    ) -> Result<String, BillingError>;

    async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        metadata: CheckoutMetadata,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSessionResult, BillingError>;

    async fn cancel_subscription(&self, stripe_subscription_id: &str) -> Result<(), BillingError>;

    async fn delete_customer(&self, stripe_customer_id: &str) -> Result<(), BillingError>;

    /// Ensures a Stripe product exists with the given stable id, upserting its
    /// mutable fields (name, description, active).
    ///
    /// Products are mutable in Stripe, so this creates the product when absent
    /// and updates its name/description when present. The `id` is a stable,
    /// caller-controlled identifier (e.g. derived from the service/plan slug)
    /// that makes the operation idempotent.
    ///
    /// Returns the Stripe product id.
    async fn ensure_product(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<String, BillingError>;

    /// Ensures a recurring price exists for the given [`PriceSpec`], converging
    /// on its stable `lookup_key`.
    ///
    /// Stripe prices are immutable for amount/currency/interval but mutable for
    /// nickname/metadata/active. Convergence therefore works as follows:
    /// - no active price carries the `lookup_key` -> create a new price with it
    ///   (tagged `managed_by`);
    /// - an active price carries the `lookup_key` and amount/currency match ->
    ///   reuse it, updating the nickname in place if it changed (no recreation);
    /// - an active price carries the `lookup_key` but amount/currency differs ->
    ///   create a new price with `transfer_lookup_key = true` (moving the key off
    ///   the old price), then retire the old price with
    ///   [`StripeClient::delete_or_archive_price`].
    ///
    /// Idempotent and concurrency-safe: the `lookup_key` can only be held by a
    /// single active price at a time.
    async fn ensure_price(&self, spec: &PriceSpec) -> Result<EnsurePriceResult, BillingError>;

    /// Retires a price by deleting it if never used, otherwise archiving it.
    ///
    /// Stripe forbids deleting a price used on an invoice, so this tries `DELETE`
    /// first (cleaning up never-used prices, e.g. mistakes) and falls back to
    /// archiving (`active = false`) when Stripe rejects the delete. Existing
    /// subscriptions are unaffected and no billing history is lost.
    async fn delete_or_archive_price(&self, price_id: &str) -> Result<(), BillingError>;

    /// Archives a product (`active = false`). Existing prices/subscriptions are
    /// unaffected. Used when a whole plan/service is removed from the catalogue.
    async fn archive_product(&self, product_id: &str) -> Result<(), BillingError>;

    /// Lists all active prices tagged `managed_by = france-nuage-catalog`.
    ///
    /// This is the pruning perimeter: only catalogue-owned prices are returned,
    /// so pruning never considers unrelated Stripe data. Stripe's list API does
    /// not filter by metadata, so implementations page through active prices and
    /// filter client-side.
    async fn list_managed_prices(&self) -> Result<Vec<ManagedPrice>, BillingError>;

    /// Lists all active products tagged `managed_by = france-nuage-catalog`.
    async fn list_managed_products(&self) -> Result<Vec<ManagedProduct>, BillingError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutMetadata {
    pub subscription_id: Uuid,
    pub plan_id: Uuid,
    pub version_id: Uuid,
    pub project_slug: String,
    pub organization_slug: String,
}

#[derive(Debug, Clone)]
pub struct CheckoutSessionResult {
    pub session_id: String,
    pub url: String,
}

#[derive(Clone)]
pub struct Billing<A: Authorize, S: StripeClient> {
    pub(crate) db: Pool<Postgres>,
    pub(crate) stripe: S,
    pub(crate) managed: ManagedServices<A>,
    pub(crate) kek: Arc<Kek>,
    pub(crate) success_url: String,
    pub(crate) cancel_url: String,
}

impl<A: Authorize, S: StripeClient> Billing<A, S> {
    pub fn new(
        db: Pool<Postgres>,
        stripe: S,
        managed: ManagedServices<A>,
        kek: Arc<Kek>,
        success_url: String,
        cancel_url: String,
    ) -> Self {
        Self {
            db,
            stripe,
            managed,
            kek,
            success_url,
            cancel_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_interval_maps_from_billing_period() {
        // Arrange & Act & Assert
        assert_eq!(
            PriceInterval::from(BillingPeriod::Monthly),
            PriceInterval::Month
        );
        assert_eq!(
            PriceInterval::from(BillingPeriod::Yearly),
            PriceInterval::Year
        );
    }
}
