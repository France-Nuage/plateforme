//! Stripe webhook event dispatch and idempotency.

use chrono::{DateTime, Utc};
use fabrique::Query;
use frn_crypto::EnvelopeCiphertext;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::authorization::Authorize;
use crate::billing::checkout::build_pending_aad;
use crate::billing::subscription::validate_status_transition;
use crate::billing::{
    Billing, BillingError, BillingSubscription, PendingInstanceParams, ProcessedEvent,
    StripeClient, SubscriptionStatus,
};
use crate::managed::{CreateInstanceRequest, DeployManagedServiceParams};
use crate::workflow::WorkflowScheduler;

#[derive(Debug, Clone)]
pub struct StripeWebhookEvent {
    pub event_id: String,
    pub event_type: String,
    pub checkout_session_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub period_start: Option<i64>,
    pub period_end: Option<i64>,
}

impl<A: Authorize, S: StripeClient> Billing<A, S> {
    /// Atomically claims a Stripe event for processing (idempotency).
    ///
    /// Inserts the event id with `ON CONFLICT DO NOTHING`. The primary-key
    /// unique constraint serialises concurrent deliveries of the same event, so
    /// exactly one caller sees the row inserted and gets it back via
    /// `RETURNING`. Returns `true` when this call won the claim and must process
    /// the event, `false` when it was already claimed and should be skipped.
    ///
    /// Claiming up front (rather than checking existence then marking at the
    /// end) closes the race where two identical deliveries both pass a prior
    /// existence check and both process the event. `DO NOTHING` raises no error
    /// on conflict, so the surrounding transaction stays usable; when processing
    /// later fails, the whole transaction (claim included) rolls back and Stripe
    /// safely retries.
    async fn claim_event(
        &self,
        conn: &mut PgConnection,
        event_id: &str,
        event_type: &str,
    ) -> Result<bool, BillingError> {
        let claimed = ProcessedEvent::query()
            .insert()
            .set(ProcessedEvent::EVENT_ID, event_id.to_owned())
            .set(ProcessedEvent::EVENT_TYPE, event_type.to_owned())
            .on_conflict()
            .do_nothing()
            .returning()
            .first(&mut *conn)
            .await?;
        Ok(claimed.is_some())
    }

    /// Dispatches a verified Stripe webhook event to the appropriate handler.
    pub async fn dispatch_webhook_event<W>(
        &self,
        conn: &mut PgConnection,
        scheduler: &W,
        event: StripeWebhookEvent,
    ) -> Result<(), BillingError>
    where
        W: WorkflowScheduler<DeployManagedServiceParams>,
    {
        if !self
            .claim_event(&mut *conn, &event.event_id, &event.event_type)
            .await?
        {
            tracing::debug!(event_id = %event.event_id, "duplicate event, skipping");
            return Ok(());
        }

        match event.event_type.as_str() {
            "checkout.session.completed" => {
                self.handle_checkout_completed(&mut *conn, scheduler, &event)
                    .await?;
            }
            "checkout.session.expired" => {
                self.handle_checkout_expired(&mut *conn, &event).await?;
            }
            "invoice.paid" => {
                self.handle_invoice_paid(&mut *conn, &event).await?;
            }
            "invoice.payment_failed" => {
                self.handle_subscription_status_change(
                    &mut *conn,
                    &event,
                    SubscriptionStatus::PastDue,
                )
                .await?;
            }
            "customer.subscription.deleted" => {
                self.handle_subscription_status_change(
                    &mut *conn,
                    &event,
                    SubscriptionStatus::Canceled,
                )
                .await?;
            }
            _ => {
                tracing::debug!(event_type = %event.event_type, "unhandled event type");
            }
        }

        tracing::info!(
            event_id = %event.event_id,
            event_type = %event.event_type,
            "webhook event processed"
        );

        Ok(())
    }

    async fn handle_checkout_completed<W>(
        &self,
        conn: &mut PgConnection,
        scheduler: &W,
        event: &StripeWebhookEvent,
    ) -> Result<(), BillingError>
    where
        W: WorkflowScheduler<DeployManagedServiceParams>,
    {
        let Some(ref session_id) = event.checkout_session_id else {
            tracing::warn!(event_id = %event.event_id, "checkout.session.completed missing checkout_session_id");
            return Ok(());
        };
        let Some(ref stripe_sub_id) = event.stripe_subscription_id else {
            tracing::warn!(event_id = %event.event_id, "checkout.session.completed missing stripe_subscription_id");
            return Ok(());
        };

        // A checkout session unknown to this instance is not an error: Stripe
        // fans every event out to all listeners connected to the same account,
        // so a control plane routinely receives sessions created by another
        // (e.g. a concurrent ephemeral environment sharing the sandbox). Ack it
        // as a no-op so Stripe marks it delivered instead of retrying forever.
        let Some(subscription) = BillingSubscription::query()
            .select()
            .r#where(
                BillingSubscription::STRIPE_CHECKOUT_SESSION_ID,
                "=",
                Some(session_id.to_owned()),
            )
            .first(&mut *conn)
            .await?
        else {
            tracing::debug!(
                event_id = %event.event_id,
                checkout_session_id = %session_id,
                "checkout session not found locally, ignoring event (not for this instance)"
            );
            return Ok(());
        };

        if subscription.status != SubscriptionStatus::PendingPayment {
            return Err(BillingError::InvalidStatusTransition {
                from: subscription.status,
                to: SubscriptionStatus::Active,
            });
        }

        BillingSubscription::query()
            .update()
            .set(BillingSubscription::STATUS, SubscriptionStatus::Active)
            .set(
                BillingSubscription::STRIPE_SUBSCRIPTION_ID,
                Some(stripe_sub_id.to_owned()),
            )
            .r#where(BillingSubscription::ID, "=", subscription.id)
            .execute(&mut *conn)
            .await?;

        let params = PendingInstanceParams::query()
            .select()
            .r#where(PendingInstanceParams::SUBSCRIPTION_ID, "=", subscription.id)
            .first(&mut *conn)
            .await?
            .ok_or(BillingError::PendingParamsNotFound(subscription.id))?;

        let secret_values: Option<Value> = match (
            &params.secret_ciphertext,
            &params.secret_nonce,
            &params.secret_dek_ciphertext,
            &params.secret_dek_nonce,
            params.secret_key_version,
        ) {
            (Some(ct), Some(nonce), Some(dek_ct), Some(dek_nonce), Some(kv)) => {
                let envelope = EnvelopeCiphertext {
                    ciphertext: ct.clone(),
                    nonce: nonce.clone(),
                    dek_ciphertext: dek_ct.clone(),
                    dek_nonce: dek_nonce.clone(),
                    key_version: kv,
                };
                let aad = build_pending_aad(subscription.id, kv);
                let plaintext = frn_crypto::decrypt(&self.kek, &envelope, &aad)?;
                Some(serde_json::from_slice(&plaintext)?)
            }
            _ => None,
        };

        let create_request = CreateInstanceRequest {
            project_slug: params.project_slug,
            organization_slug: params.organization_slug,
            service_slug: params.service_slug,
            version_id: params.version_id,
            plan_id: subscription.plan_id,
            user_values: params.user_values,
            secret_values,
        };

        let instance = self
            .managed
            .create_instance_unchecked(&mut *conn, scheduler, create_request)
            .await?;

        BillingSubscription::query()
            .update()
            .set(BillingSubscription::INSTANCE_ID, Some(instance.id))
            .r#where(BillingSubscription::ID, "=", subscription.id)
            .execute(&mut *conn)
            .await?;

        Self::delete_pending_instance_params(&mut *conn, subscription.id).await?;

        tracing::info!(
            subscription_id = %subscription.id,
            instance_id = %instance.id,
            stripe_subscription_id = stripe_sub_id,
            "checkout completed, subscription activated and instance provisioning started"
        );

        Ok(())
    }

    async fn handle_checkout_expired(
        &self,
        conn: &mut PgConnection,
        event: &StripeWebhookEvent,
    ) -> Result<(), BillingError> {
        let Some(ref session_id) = event.checkout_session_id else {
            tracing::warn!(event_id = %event.event_id, "checkout.session.expired missing checkout_session_id");
            return Ok(());
        };

        let subscription = BillingSubscription::query()
            .select()
            .r#where(
                BillingSubscription::STRIPE_CHECKOUT_SESSION_ID,
                "=",
                Some(session_id.to_owned()),
            )
            .first(&mut *conn)
            .await?;

        if let Some(sub) = subscription {
            BillingSubscription::query()
                .update()
                .set(BillingSubscription::STATUS, SubscriptionStatus::Canceled)
                .r#where(BillingSubscription::ID, "=", sub.id)
                .execute(&mut *conn)
                .await?;

            Self::delete_pending_instance_params(&mut *conn, sub.id).await?;

            tracing::info!(
                subscription_id = %sub.id,
                "checkout expired, subscription canceled and pending params cleaned"
            );
        }

        Ok(())
    }

    async fn handle_invoice_paid(
        &self,
        conn: &mut PgConnection,
        event: &StripeWebhookEvent,
    ) -> Result<(), BillingError> {
        let Some(ref stripe_sub_id) = event.stripe_subscription_id else {
            tracing::warn!(event_id = %event.event_id, "invoice.paid missing stripe_subscription_id");
            return Ok(());
        };

        if let (Some(start), Some(end)) = (event.period_start, event.period_end) {
            let period_start = DateTime::<Utc>::from_timestamp(start, 0)
                .ok_or(BillingError::InvalidTimestamp(start))?;
            let period_end = DateTime::<Utc>::from_timestamp(end, 0)
                .ok_or(BillingError::InvalidTimestamp(end))?;

            let Some(subscription) =
                Self::find_subscription_by_stripe_id_on(&mut *conn, stripe_sub_id).await?
            else {
                tracing::debug!(
                    event_id = %event.event_id,
                    stripe_subscription_id = %stripe_sub_id,
                    "subscription not found locally, ignoring invoice.paid (not for this instance)"
                );
                return Ok(());
            };

            BillingSubscription::query()
                .update()
                .set(
                    BillingSubscription::CURRENT_PERIOD_START,
                    Some(period_start),
                )
                .set(BillingSubscription::CURRENT_PERIOD_END, Some(period_end))
                .r#where(BillingSubscription::ID, "=", subscription.id)
                .execute(&mut *conn)
                .await?;
        }

        Ok(())
    }

    async fn handle_subscription_status_change(
        &self,
        conn: &mut PgConnection,
        event: &StripeWebhookEvent,
        new_status: SubscriptionStatus,
    ) -> Result<(), BillingError> {
        let Some(ref stripe_sub_id) = event.stripe_subscription_id else {
            tracing::warn!(
                event_id = %event.event_id,
                event_type = %event.event_type,
                "event missing stripe_subscription_id"
            );
            return Ok(());
        };

        let Some(subscription) =
            Self::find_subscription_by_stripe_id_on(&mut *conn, stripe_sub_id).await?
        else {
            tracing::debug!(
                event_id = %event.event_id,
                event_type = %event.event_type,
                stripe_subscription_id = %stripe_sub_id,
                "subscription not found locally, ignoring status change (not for this instance)"
            );
            return Ok(());
        };

        validate_status_transition(subscription.status, new_status)?;

        BillingSubscription::query()
            .update()
            .set(BillingSubscription::STATUS, new_status)
            .r#where(BillingSubscription::ID, "=", subscription.id)
            .execute(&mut *conn)
            .await?;

        tracing::info!(
            subscription_id = %subscription.id,
            stripe_subscription_id = stripe_sub_id,
            old_status = %subscription.status,
            new_status = %new_status,
            "subscription status updated"
        );

        Ok(())
    }

    /// Looks up a subscription by its Stripe id, returning `None` when this
    /// instance does not own it.
    ///
    /// A missing subscription is not an error for webhook handling: Stripe
    /// broadcasts each event to every listener on the account, so events for
    /// subscriptions owned by another environment (sharing the same sandbox)
    /// are expected. Callers ignore `None` and ack the event rather than
    /// returning a 500 that would make Stripe retry indefinitely.
    async fn find_subscription_by_stripe_id_on(
        conn: &mut PgConnection,
        stripe_subscription_id: &str,
    ) -> Result<Option<BillingSubscription>, BillingError> {
        Ok(BillingSubscription::query()
            .select()
            .r#where(
                BillingSubscription::STRIPE_SUBSCRIPTION_ID,
                "=",
                Some(stripe_subscription_id.to_owned()),
            )
            .first(&mut *conn)
            .await?)
    }

    // fabrique limitation: DELETE with specific WHERE not supported via builder
    async fn delete_pending_instance_params(
        conn: &mut PgConnection,
        subscription_id: Uuid,
    ) -> Result<(), BillingError> {
        sqlx::query("DELETE FROM billing.pending_instance_params WHERE subscription_id = $1")
            .bind(subscription_id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }
}
