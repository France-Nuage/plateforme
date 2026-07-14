//! Stripe webhook HTTP handler.
//!
//! Exposes `POST /webhooks/stripe` outside the gRPC service layer.
//! Verifies the Stripe signature via `async-stripe-webhook`, then
//! converts the typed event into a `StripeWebhookEvent` DTO for
//! dispatch by the billing service.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use sqlx::{Pool, Postgres};
use stripe_webhook::{EventObject, Webhook};
use workflow::scheduler::ManagedWorkflowScheduler;

use frn_core::authorization::Authorize;
use frn_core::billing::{Billing, StripeClient, StripeWebhookEvent};

#[derive(Clone)]
pub struct WebhookState<A: Authorize, S: StripeClient> {
    pub billing: Billing<A, S>,
    pub pool: Pool<Postgres>,
    pub webhook_secret: Arc<String>,
}

pub async fn stripe_webhook_handler<A: Authorize + 'static, S: StripeClient + 'static>(
    State(state): State<WebhookState<A, S>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let signature = match headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(sig) => sig.to_owned(),
        None => {
            tracing::warn!("webhook request missing Stripe-Signature header");
            return StatusCode::BAD_REQUEST;
        }
    };

    let payload = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "webhook body is not valid UTF-8");
            return StatusCode::BAD_REQUEST;
        }
    };

    let event = match Webhook::construct_event(payload, &signature, &state.webhook_secret) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, "webhook signature verification failed");
            return StatusCode::BAD_REQUEST;
        }
    };

    let event_id = event.id.to_string();
    let event_type = event.type_.to_string();
    let webhook_event = extract_webhook_event(&event_id, &event_type, &event.data.object);

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "failed to begin db transaction for webhook");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let scheduler = ManagedWorkflowScheduler;

    match state
        .billing
        .dispatch_webhook_event(&mut tx, &scheduler, webhook_event)
        .await
    {
        Ok(()) => {
            if let Err(e) = tx.commit().await {
                tracing::error!(error = %e, "failed to commit webhook transaction");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!(
                event_id = %event_id,
                event_type = %event_type,
                error = %e,
                "webhook event processing failed"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn extract_webhook_event(
    event_id: &str,
    event_type: &str,
    object: &EventObject,
) -> StripeWebhookEvent {
    match object {
        EventObject::CheckoutSessionCompleted(session)
        | EventObject::CheckoutSessionExpired(session) => {
            let subscription_id = session.subscription.as_ref().map(|s| s.id().to_string());
            StripeWebhookEvent {
                event_id: event_id.to_owned(),
                event_type: event_type.to_owned(),
                checkout_session_id: Some(session.id.to_string()),
                stripe_subscription_id: subscription_id,
                period_start: None,
                period_end: None,
            }
        }
        EventObject::InvoicePaid(invoice) | EventObject::InvoicePaymentFailed(invoice) => {
            let subscription_id = invoice.subscription.as_ref().map(|s| s.id().to_string());
            StripeWebhookEvent {
                event_id: event_id.to_owned(),
                event_type: event_type.to_owned(),
                checkout_session_id: None,
                stripe_subscription_id: subscription_id,
                period_start: Some(invoice.period_start),
                period_end: Some(invoice.period_end),
            }
        }
        EventObject::CustomerSubscriptionDeleted(sub)
        | EventObject::CustomerSubscriptionUpdated(sub) => StripeWebhookEvent {
            event_id: event_id.to_owned(),
            event_type: event_type.to_owned(),
            checkout_session_id: None,
            stripe_subscription_id: Some(sub.id.to_string()),
            period_start: None,
            period_end: None,
        },
        _ => StripeWebhookEvent {
            event_id: event_id.to_owned(),
            event_type: event_type.to_owned(),
            checkout_session_id: None,
            stripe_subscription_id: None,
            period_start: None,
            period_end: None,
        },
    }
}
