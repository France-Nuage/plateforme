//! Production Stripe API client using async-stripe.

use std::collections::HashMap;

use stripe::Client;
use stripe_billing::subscription::CancelSubscription;
use stripe_checkout::checkout_session::{CreateCheckoutSession, CreateCheckoutSessionLineItems};
use stripe_core::customer::{CreateCustomer, DeleteCustomer};
use stripe_product::price::{
    CreatePrice, CreatePriceRecurring, CreatePriceRecurringInterval, ListPrice, UpdatePrice,
};
use stripe_product::product::{CreateProduct, ListProduct, RetrieveProduct, UpdateProduct};
use stripe_shared::CheckoutSessionMode;

use futures::StreamExt;

use crate::billing::{
    BillingError, CATALOG_MANAGED_BY_KEY, CATALOG_MANAGED_BY_VALUE, CheckoutMetadata,
    CheckoutSessionResult, EnsurePriceResult, ManagedPrice, ManagedProduct, PriceInterval,
    PriceSpec, StripeClient,
};

/// Builds the `managed_by` metadata marking an object as catalogue-owned.
fn managed_by_metadata() -> HashMap<String, String> {
    HashMap::from([(
        CATALOG_MANAGED_BY_KEY.to_owned(),
        CATALOG_MANAGED_BY_VALUE.to_owned(),
    )])
}

impl From<PriceInterval> for CreatePriceRecurringInterval {
    fn from(interval: PriceInterval) -> Self {
        match interval {
            PriceInterval::Month => CreatePriceRecurringInterval::Month,
            PriceInterval::Year => CreatePriceRecurringInterval::Year,
        }
    }
}

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

    async fn ensure_product(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<String, BillingError> {
        // Products are keyed by a stable, caller-controlled id. Retrieve first
        // to decide between create (absent) and update (present); this avoids
        // depending on Stripe's error-code shape to detect existence. Every
        // product is tagged managed_by so pruning stays within our perimeter.
        let exists = RetrieveProduct::new(id).send(&self.client).await.is_ok();

        if exists {
            let mut update = UpdateProduct::new(id)
                .name(name)
                .active(true)
                .metadata(managed_by_metadata());
            if let Some(description) = description {
                update = update.description(description);
            }
            let product = update
                .send(&self.client)
                .await
                .map_err(|e| BillingError::Stripe(e.to_string()))?;
            Ok(product.id.to_string())
        } else {
            let mut create = CreateProduct::new(name)
                .id(id)
                .active(true)
                .metadata(managed_by_metadata());
            if let Some(description) = description {
                create = create.description(description);
            }
            let product = create
                .send(&self.client)
                .await
                .map_err(|e| BillingError::Stripe(e.to_string()))?;
            Ok(product.id.to_string())
        }
    }

    async fn ensure_price(&self, spec: &PriceSpec) -> Result<EnsurePriceResult, BillingError> {
        let currency: stripe_types::Currency = spec
            .currency
            .parse()
            .map_err(|_| BillingError::Stripe(format!("invalid currency: {}", spec.currency)))?;

        // Find the active price currently carrying this lookup_key, if any.
        let existing = ListPrice::new()
            .lookup_keys(vec![spec.lookup_key.clone()])
            .active(true)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        let current = existing.data.into_iter().next();

        // Reuse when an active price with the same amount/currency already holds
        // the key. Amount/currency are immutable, but the nickname is not: update
        // it in place if it changed, without recreating the price.
        if let Some(price) = &current {
            let amount_matches = price.unit_amount == Some(spec.unit_amount_cents);
            let currency_matches = price.currency == currency;
            if amount_matches && currency_matches {
                if price.nickname.as_deref() != spec.nickname.as_deref() {
                    let mut update = UpdatePrice::new(price.id.as_str());
                    if let Some(nickname) = &spec.nickname {
                        update = update.nickname(nickname.clone());
                    }
                    update
                        .send(&self.client)
                        .await
                        .map_err(|e| BillingError::Stripe(e.to_string()))?;
                }
                return Ok(EnsurePriceResult {
                    price_id: price.id.to_string(),
                    created: false,
                });
            }
        }

        // Amount/currency changed (or first creation): create a new price
        // carrying the lookup_key. When a stale price holds the key,
        // transfer_lookup_key moves it onto the new price.
        let mut create = CreatePrice::new(currency)
            .product(spec.product_id.clone())
            .unit_amount(spec.unit_amount_cents)
            .lookup_key(spec.lookup_key.clone())
            .metadata(managed_by_metadata());
        // Recurring price when an interval is set; otherwise a one-time price.
        if let Some(interval) = spec.interval {
            create = create.recurring(CreatePriceRecurring::new(
                CreatePriceRecurringInterval::from(interval),
            ));
        }
        if let Some(nickname) = &spec.nickname {
            create = create.nickname(nickname.clone());
        }
        if current.is_some() {
            create = create.transfer_lookup_key(true);
        }

        let new_price = create
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        // Retire the superseded price. Existing subscriptions remain active.
        if let Some(old) = current {
            self.delete_or_archive_price(old.id.as_str()).await?;
        }

        Ok(EnsurePriceResult {
            price_id: new_price.id.to_string(),
            created: true,
        })
    }

    async fn delete_or_archive_price(&self, price_id: &str) -> Result<(), BillingError> {
        // Stripe has no delete-price API (prices used on an invoice can never be
        // deleted, and unused ones can only be removed from the Dashboard), so
        // the API-safe retirement is to archive. Existing subscriptions keep
        // their price; the price just can't be used for new purchases.
        UpdatePrice::new(price_id)
            .active(false)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        Ok(())
    }

    async fn archive_product(&self, product_id: &str) -> Result<(), BillingError> {
        UpdateProduct::new(product_id)
            .active(false)
            .send(&self.client)
            .await
            .map_err(|e| BillingError::Stripe(e.to_string()))?;

        Ok(())
    }

    async fn list_managed_prices(&self) -> Result<Vec<ManagedPrice>, BillingError> {
        // Stripe's list API cannot filter by metadata, so page through active
        // prices and keep only those tagged managed_by.
        let mut stream = ListPrice::new()
            .active(true)
            .limit(100)
            .paginate()
            .stream(&self.client);

        let mut prices = Vec::new();
        while let Some(item) = stream.next().await {
            let price = item.map_err(|e| BillingError::Stripe(e.to_string()))?;
            if is_catalog_managed(&price.metadata) {
                prices.push(ManagedPrice {
                    id: price.id.to_string(),
                    lookup_key: price.lookup_key.clone(),
                });
            }
        }
        Ok(prices)
    }

    async fn list_managed_products(&self) -> Result<Vec<ManagedProduct>, BillingError> {
        let mut stream = ListProduct::new()
            .active(true)
            .limit(100)
            .paginate()
            .stream(&self.client);

        let mut products = Vec::new();
        while let Some(item) = stream.next().await {
            let product = item.map_err(|e| BillingError::Stripe(e.to_string()))?;
            if is_catalog_managed(&product.metadata) {
                products.push(ManagedProduct {
                    id: product.id.to_string(),
                });
            }
        }
        Ok(products)
    }
}

/// Returns whether a Stripe object's metadata marks it as catalogue-owned.
fn is_catalog_managed(metadata: &HashMap<String, String>) -> bool {
    metadata.get(CATALOG_MANAGED_BY_KEY).map(String::as_str) == Some(CATALOG_MANAGED_BY_VALUE)
}
