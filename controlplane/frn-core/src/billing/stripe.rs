//! Production Stripe API client using async-stripe.

use std::collections::HashMap;

use stripe::Client;
use stripe_billing::subscription::CancelSubscription;
use stripe_checkout::checkout_session::{CreateCheckoutSession, CreateCheckoutSessionLineItems};
use stripe_core::customer::{CreateCustomer, DeleteCustomer};
use stripe_shared::CheckoutSessionMode;

use crate::billing::{BillingError, CheckoutMetadata, CheckoutSessionResult, StripeClient};

#[derive(Clone)]
pub struct HttpStripeClient {
    client: Client,
}

impl HttpStripeClient {
    pub fn new(secret_key: String) -> Self {
        let client = Client::new(secret_key);
        Self { client }
    }
}

impl StripeClient for HttpStripeClient {
    async fn create_customer(
        &self,
        organization_slug: &str,
        organization_name: &str,
    ) -> Result<String, BillingError> {
        let metadata =
            HashMap::from([("organization_slug".to_owned(), organization_slug.to_owned())]);

        let customer = CreateCustomer::new()
            .name(organization_name)
            .metadata(metadata)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        Ok(customer.id.to_string())
    }

    async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        metadata: CheckoutMetadata,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSessionResult, BillingError> {
        let meta = HashMap::from([
            (
                "subscription_id".to_owned(),
                metadata.subscription_id.to_string(),
            ),
            ("plan_id".to_owned(), metadata.plan_id.to_string()),
            ("version_id".to_owned(), metadata.version_id.to_string()),
            ("project_slug".to_owned(), metadata.project_slug),
            ("organization_slug".to_owned(), metadata.organization_slug),
        ]);

        let line_item = CreateCheckoutSessionLineItems {
            price: Some(price_id.to_owned()),
            quantity: Some(1),
            ..Default::default()
        };

        let session = CreateCheckoutSession::new()
            .customer(customer_id)
            .mode(CheckoutSessionMode::Subscription)
            .line_items(vec![line_item])
            .success_url(success_url)
            .cancel_url(cancel_url)
            .metadata(meta)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        let url = session
            .url
            .ok_or_else(|| BillingError::Stripe("checkout session has no URL".to_owned()))?;

        Ok(CheckoutSessionResult {
            session_id: session.id.to_string(),
            url,
        })
    }

    async fn cancel_subscription(&self, stripe_subscription_id: &str) -> Result<(), BillingError> {
        CancelSubscription::new(stripe_subscription_id)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        Ok(())
    }

    async fn delete_customer(&self, stripe_customer_id: &str) -> Result<(), BillingError> {
        DeleteCustomer::new(stripe_customer_id)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        Ok(())
    }
}
