//! Billing and subscription management for managed services.
//!
//! Provides entity definitions, Stripe integration traits, and service layers
//! for handling checkout sessions, subscriptions, and webhook processing.

mod checkout;
mod customer;
pub mod stripe;
mod subscription;
mod webhook;

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

/// Stripe API abstraction for testability.
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
