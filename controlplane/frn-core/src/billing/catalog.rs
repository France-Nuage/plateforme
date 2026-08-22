//! Catalogue reconciliation: pushes the declarative catalogue into Stripe.
//!
//! Turns the parsed [`Catalog`] into Stripe products and prices. Every product
//! and price is tagged `managed_by = france-nuage-catalog`, so the reconciler
//! owns a well-defined perimeter and never touches unrelated Stripe data. All
//! operations are idempotent and keyed by stable identifiers (product id and
//! price `lookup_key`), so re-running — including after an interrupted run —
//! converges on the same state without creating duplicates.
//!
//! The reconciler returns a map from `lookup_key` to the resulting Stripe
//! `price_...` id, which the caller uses to persist price ids on managed-service
//! plans in the database.

use std::collections::HashMap;

use serde_json::Value;

use crate::authorization::Authorize;
use crate::billing::{Billing, BillingError, PriceSpec, StripeClient};
use crate::managed::{
    BillableProduct, Catalog, CatalogInterval, CatalogPlan, CatalogPrice, ManagedServiceEntry,
    PlanEntitlement,
};

/// Outcome of reconciling the catalogue into Stripe.
///
/// Maps each declared `lookup_key` to the Stripe price id that now carries it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconciledCatalog {
    /// `lookup_key` -> resulting Stripe `price_...` id.
    pub price_ids: HashMap<String, String>,
}

impl ReconciledCatalog {
    /// Returns the Stripe price id for a declared lookup key, if reconciled.
    pub fn price_id(&self, lookup_key: &str) -> Option<&str> {
        self.price_ids.get(lookup_key).map(String::as_str)
    }
}

/// A catalogue-owned Stripe price found during pruning that is no longer
/// declared in the catalogue (an orphan to retire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanPrice {
    pub id: String,
    pub lookup_key: Option<String>,
}

/// A catalogue-owned Stripe product found during pruning that is no longer
/// declared in the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanProduct {
    pub id: String,
}

/// Result of a prune run.
///
/// Lists the catalogue-owned Stripe objects that are no longer declared in the
/// catalogue. When the run is a dry run, `archived` is `false` and nothing was
/// modified; otherwise the listed objects were archived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PruneReport {
    /// Orphan prices (tagged managed_by, lookup key absent from the catalogue).
    pub prices: Vec<OrphanPrice>,
    /// Orphan products (tagged managed_by, id absent from the catalogue).
    pub products: Vec<OrphanProduct>,
    /// Whether the orphans were actually archived (`false` for a dry run).
    pub archived: bool,
}

impl PruneReport {
    /// Returns whether any orphan was found.
    pub fn is_empty(&self) -> bool {
        self.prices.is_empty() && self.products.is_empty()
    }
}

impl<A: Authorize, S: StripeClient> Billing<A, S> {
    /// Syncs the catalogue into Stripe and the database.
    ///
    /// First reconciles the whole catalogue into Stripe (products + prices),
    /// then persists the managed services and their plans into the database,
    /// wiring each plan to the Stripe price ids just reconciled. Resources and
    /// legacy products are reconciled into Stripe only (they are not deployable
    /// services and are not stored as such).
    ///
    /// # Errors
    /// Returns [`BillingError`] on any Stripe or database failure.
    pub async fn sync_catalog(&self, catalog: &Catalog) -> Result<(), BillingError> {
        let reconciled = self.reconcile_catalog(catalog).await?;
        self.persist_managed_services(catalog, &reconciled).await?;
        Ok(())
    }

    /// Prunes catalogue-owned Stripe objects that are no longer declared.
    ///
    /// Lists the prune perimeter (products/prices tagged `managed_by`), finds
    /// those absent from the catalogue, and — unless `dry_run` — archives them.
    /// Prices are archived (Stripe forbids deleting used prices); existing
    /// subscriptions are unaffected. Only tagged objects are ever considered, so
    /// unrelated Stripe data is never touched.
    ///
    /// Returns a [`PruneReport`] listing the orphans (and whether they were
    /// archived). With `dry_run = true`, nothing is modified.
    ///
    /// # Errors
    /// Returns [`BillingError::Stripe`] on any Stripe failure.
    pub async fn prune_catalog(
        &self,
        catalog: &Catalog,
        dry_run: bool,
    ) -> Result<PruneReport, BillingError> {
        let declared_lookup_keys = catalog.all_lookup_keys();
        let declared_product_ids = catalog.all_stripe_product_ids();

        let managed_prices = self.stripe.list_managed_prices().await?;
        let managed_products = self.stripe.list_managed_products().await?;

        let orphan_prices = orphan_prices(&managed_prices, &declared_lookup_keys);
        let orphan_products = orphan_products(&managed_products, &declared_product_ids);

        if !dry_run {
            // Archive prices before products (a product with active prices can't
            // be archived cleanly otherwise).
            for price in &orphan_prices {
                self.stripe.delete_or_archive_price(&price.id).await?;
            }
            for product in &orphan_products {
                self.stripe.archive_product(&product.id).await?;
            }
        }

        Ok(PruneReport {
            prices: orphan_prices,
            products: orphan_products,
            archived: !dry_run,
        })
    }

    /// Persists managed services and their plans into the database.
    async fn persist_managed_services(
        &self,
        catalog: &Catalog,
        reconciled: &ReconciledCatalog,
    ) -> Result<(), BillingError> {
        let mut tx = self.managed.begin().await?;

        for service in &catalog.managed_services {
            let stored = self
                .managed
                .upsert_service(
                    &mut tx,
                    &service.slug,
                    &service.name,
                    service.description.as_deref(),
                    service.category.clone(),
                    service.database_engine.clone(),
                    service.deploy_target.as_ref(),
                )
                .await?;

            for plan in &service.plans {
                let (monthly, yearly) = plan_price_ids(plan, reconciled);
                let entitlements = plan_entitlements_json(plan)?;
                self.managed
                    .upsert_plan(
                        &mut tx,
                        stored.id,
                        &plan.slug,
                        &plan.name,
                        plan.description.as_deref(),
                        &plan.status,
                        plan.highlighted,
                        plan.values_override.as_ref(),
                        &entitlements,
                        plan_amount(plan, CatalogInterval::Month),
                        plan_amount(plan, CatalogInterval::Year),
                        monthly.as_deref(),
                        yearly.as_deref(),
                        plan.requires_payment,
                    )
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Reconciles the whole catalogue into Stripe (products + prices).
    ///
    /// Processes all three sections (managed services, resources, legacy):
    /// every entry maps to its existing Stripe product (tagged `managed_by`),
    /// and every declared price is ensured under its explicit `lookup_key`.
    ///
    /// Returns the `lookup_key -> price_id` map for DB persistence. Does not
    /// prune.
    ///
    /// # Errors
    /// Returns [`BillingError::Stripe`] if any Stripe operation fails.
    pub async fn reconcile_catalog(
        &self,
        catalog: &Catalog,
    ) -> Result<ReconciledCatalog, BillingError> {
        let mut result = ReconciledCatalog::default();

        for service in &catalog.managed_services {
            self.reconcile_managed_service(service, &mut result).await?;
        }
        for product in catalog.resources.iter().chain(&catalog.legacy) {
            self.reconcile_billable_product(product, &mut result)
                .await?;
        }

        Ok(result)
    }

    /// Reconciles one managed service: a single Stripe product, and one price
    /// per plan price (tiers/periods).
    async fn reconcile_managed_service(
        &self,
        service: &ManagedServiceEntry,
        result: &mut ReconciledCatalog,
    ) -> Result<(), BillingError> {
        let product_id = self
            .stripe
            .ensure_product(
                &service.stripe_product_id,
                &service.name,
                service.description.as_deref(),
            )
            .await?;

        for plan in &service.plans {
            for price in &plan.prices {
                self.ensure_catalog_price(&product_id, price, result)
                    .await?;
            }
        }
        Ok(())
    }

    /// Reconciles a bare billable product (resource or legacy) and its prices.
    async fn reconcile_billable_product(
        &self,
        product: &BillableProduct,
        result: &mut ReconciledCatalog,
    ) -> Result<(), BillingError> {
        let product_id = self
            .stripe
            .ensure_product(
                &product.stripe_product_id,
                &product.name,
                product.description.as_deref(),
            )
            .await?;

        for price in &product.prices {
            self.ensure_catalog_price(&product_id, price, result)
                .await?;
        }
        Ok(())
    }

    /// Ensures one catalogue price under `product_id`, recording its price id.
    async fn ensure_catalog_price(
        &self,
        product_id: &str,
        price: &CatalogPrice,
        result: &mut ReconciledCatalog,
    ) -> Result<(), BillingError> {
        let spec = PriceSpec {
            lookup_key: price.lookup_key.clone(),
            product_id: product_id.to_owned(),
            unit_amount_cents: price.unit_amount_cents,
            currency: price.currency.clone(),
            interval: price.interval.map(Into::into),
            nickname: price.nickname.clone(),
        };

        let ensured = self.stripe.ensure_price(&spec).await?;
        result
            .price_ids
            .insert(price.lookup_key.clone(), ensured.price_id);
        Ok(())
    }
}

/// Returns the reconciled Stripe price ids for a plan's monthly and yearly
/// prices, mapping each declared price by its interval.
///
/// Projects the plan's N prices onto the two `stripe_price_id_monthly/yearly`
/// columns of `managed.service_plan`. See issue #8033 for the flexible-pricing
/// table that will supersede this projection.
fn plan_price_ids(
    plan: &CatalogPlan,
    reconciled: &ReconciledCatalog,
) -> (Option<String>, Option<String>) {
    let mut monthly = None;
    let mut yearly = None;
    for price in &plan.prices {
        let id = reconciled.price_id(&price.lookup_key).map(str::to_owned);
        match price.interval {
            Some(CatalogInterval::Month) => monthly = id,
            Some(CatalogInterval::Year) => yearly = id,
            None => {} // one-time prices are not projected onto plan columns
        }
    }
    (monthly, yearly)
}

/// Returns the plan's amount in cents for a given recurring interval, if declared.
fn plan_amount(plan: &CatalogPlan, interval: CatalogInterval) -> Option<i64> {
    plan.prices
        .iter()
        .find(|p| p.interval == Some(interval))
        .map(|p| p.unit_amount_cents)
}

/// Computes orphan prices: managed prices whose lookup key is missing or no
/// longer declared in the catalogue.
fn orphan_prices(
    managed: &[crate::billing::ManagedPrice],
    declared_lookup_keys: &std::collections::HashSet<String>,
) -> Vec<OrphanPrice> {
    managed
        .iter()
        .filter(|p| match &p.lookup_key {
            Some(key) => !declared_lookup_keys.contains(key),
            None => true,
        })
        .map(|p| OrphanPrice {
            id: p.id.clone(),
            lookup_key: p.lookup_key.clone(),
        })
        .collect()
}

/// Computes orphan products: managed products whose id is no longer declared.
fn orphan_products(
    managed: &[crate::billing::ManagedProduct],
    declared_product_ids: &std::collections::HashSet<String>,
) -> Vec<OrphanProduct> {
    managed
        .iter()
        .filter(|p| !declared_product_ids.contains(&p.id))
        .map(|p| OrphanProduct { id: p.id.clone() })
        .collect()
}

/// Serializes a plan's catalogue entitlements into the JSON shape stored on the
/// plan row.
fn plan_entitlements_json(plan: &CatalogPlan) -> Result<Value, BillingError> {
    let entitlements: Vec<PlanEntitlement> = plan
        .entitlements
        .iter()
        .map(|e| PlanEntitlement {
            key: e.key.clone(),
            label: e.label.clone(),
            value: e.value.clone(),
        })
        .collect();
    Ok(serde_json::to_value(entitlements)?)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use crate::billing::{BillingError, EnsurePriceResult, PriceInterval, PriceSpec, StripeClient};

    /// A recorded Stripe price in the fake store.
    #[derive(Debug, Clone)]
    struct FakePrice {
        id: String,
        lookup_key: String,
        product_id: String,
        unit_amount_cents: i64,
        currency: String,
        interval: Option<PriceInterval>,
        nickname: Option<String>,
        active: bool,
    }

    /// In-memory reference implementation of [`StripeClient`] modelling Stripe's
    /// product/price semantics (immutable amount, mutable nickname, lookup_key
    /// transfer, archiving). Lets us assert reconciliation behaviour without a
    /// live Stripe account.
    #[derive(Clone, Default)]
    struct FakeStripeClient {
        inner: Arc<Mutex<FakeState>>,
    }

    #[derive(Default)]
    struct FakeState {
        products: Vec<(String, String, Option<String>, bool)>, // id, name, description, active
        prices: Vec<FakePrice>,
        seq: usize,
    }

    impl FakeStripeClient {
        fn active_prices(&self) -> Vec<FakePrice> {
            self.inner
                .lock()
                .unwrap()
                .prices
                .iter()
                .filter(|p| p.active)
                .cloned()
                .collect()
        }

        fn all_prices(&self) -> Vec<FakePrice> {
            self.inner.lock().unwrap().prices.clone()
        }

        fn products(&self) -> Vec<(String, String, Option<String>, bool)> {
            self.inner.lock().unwrap().products.clone()
        }
    }

    impl StripeClient for FakeStripeClient {
        async fn create_customer(&self, _: &str, _: &str) -> Result<String, BillingError> {
            unreachable!("not used in catalogue tests")
        }

        async fn create_checkout_session(
            &self,
            _: &str,
            _: &str,
            _: crate::billing::CheckoutMetadata,
            _: &str,
            _: &str,
        ) -> Result<crate::billing::CheckoutSessionResult, BillingError> {
            unreachable!("not used in catalogue tests")
        }

        async fn cancel_subscription(&self, _: &str) -> Result<(), BillingError> {
            unreachable!("not used in catalogue tests")
        }

        async fn delete_customer(&self, _: &str) -> Result<(), BillingError> {
            unreachable!("not used in catalogue tests")
        }

        async fn ensure_product(
            &self,
            id: &str,
            name: &str,
            description: Option<&str>,
        ) -> Result<String, BillingError> {
            let mut state = self.inner.lock().unwrap();
            if let Some(product) = state.products.iter_mut().find(|p| p.0 == id) {
                product.1 = name.to_owned();
                product.2 = description.map(str::to_owned);
                product.3 = true;
            } else {
                state.products.push((
                    id.to_owned(),
                    name.to_owned(),
                    description.map(str::to_owned),
                    true,
                ));
            }
            Ok(id.to_owned())
        }

        async fn ensure_price(&self, spec: &PriceSpec) -> Result<EnsurePriceResult, BillingError> {
            let mut state = self.inner.lock().unwrap();

            let current_idx = state
                .prices
                .iter()
                .position(|p| p.active && p.lookup_key == spec.lookup_key);

            if let Some(idx) = current_idx {
                let same = state.prices[idx].unit_amount_cents == spec.unit_amount_cents
                    && state.prices[idx].currency == spec.currency;
                if same {
                    // Mutable nickname: update in place, no recreation.
                    state.prices[idx].nickname = spec.nickname.clone();
                    return Ok(EnsurePriceResult {
                        price_id: state.prices[idx].id.clone(),
                        created: false,
                    });
                }
                // Amount changed: transfer_lookup_key + retire old price.
                state.prices[idx].lookup_key = String::new();
                state.prices[idx].active = false;
            }

            state.seq += 1;
            let id = format!("price_{}", state.seq);
            state.prices.push(FakePrice {
                id: id.clone(),
                lookup_key: spec.lookup_key.clone(),
                product_id: spec.product_id.clone(),
                unit_amount_cents: spec.unit_amount_cents,
                currency: spec.currency.clone(),
                interval: spec.interval,
                nickname: spec.nickname.clone(),
                active: true,
            });

            Ok(EnsurePriceResult {
                price_id: id,
                created: true,
            })
        }

        async fn delete_or_archive_price(&self, price_id: &str) -> Result<(), BillingError> {
            let mut state = self.inner.lock().unwrap();
            if let Some(price) = state.prices.iter_mut().find(|p| p.id == price_id) {
                price.active = false;
            }
            Ok(())
        }

        async fn archive_product(&self, product_id: &str) -> Result<(), BillingError> {
            let mut state = self.inner.lock().unwrap();
            if let Some(product) = state.products.iter_mut().find(|p| p.0 == product_id) {
                product.3 = false;
            }
            Ok(())
        }

        async fn list_managed_prices(
            &self,
        ) -> Result<Vec<crate::billing::ManagedPrice>, BillingError> {
            // In the fake, every created object is catalogue-owned.
            Ok(self
                .active_prices()
                .into_iter()
                .map(|p| crate::billing::ManagedPrice {
                    id: p.id,
                    lookup_key: (!p.lookup_key.is_empty()).then_some(p.lookup_key),
                })
                .collect())
        }

        async fn list_managed_products(
            &self,
        ) -> Result<Vec<crate::billing::ManagedProduct>, BillingError> {
            Ok(self
                .products()
                .into_iter()
                .filter(|p| p.3)
                .map(|p| crate::billing::ManagedProduct { id: p.0 })
                .collect())
        }
    }

    fn spec(lookup_key: &str, amount: i64) -> PriceSpec {
        PriceSpec {
            lookup_key: lookup_key.to_owned(),
            product_id: "gitlab-runner-standard".to_owned(),
            unit_amount_cents: amount,
            currency: "eur".to_owned(),
            interval: Some(PriceInterval::Month),
            nickname: None,
        }
    }

    #[tokio::test]
    async fn ensure_price_creates_when_absent() {
        // Arrange
        let stripe = FakeStripeClient::default();

        // Act
        let result = stripe.ensure_price(&spec("k-monthly", 2500)).await.unwrap();

        // Assert
        assert!(result.created);
        let active = stripe.active_prices();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].unit_amount_cents, 2500);
        assert_eq!(active[0].product_id, "gitlab-runner-standard");
        assert_eq!(active[0].interval, Some(PriceInterval::Month));
    }

    #[tokio::test]
    async fn ensure_price_is_idempotent_when_amount_unchanged() {
        // Arrange
        let stripe = FakeStripeClient::default();
        let first = stripe.ensure_price(&spec("k-monthly", 2500)).await.unwrap();

        // Act
        let second = stripe.ensure_price(&spec("k-monthly", 2500)).await.unwrap();

        // Assert
        assert!(!second.created);
        assert_eq!(first.price_id, second.price_id);
        assert_eq!(stripe.all_prices().len(), 1);
    }

    #[tokio::test]
    async fn ensure_price_updates_nickname_without_recreating() {
        // Arrange
        let stripe = FakeStripeClient::default();
        let mut s = spec("k-monthly", 2500);
        let first = stripe.ensure_price(&s).await.unwrap();

        // Act: same amount, changed nickname.
        s.nickname = Some("pico".to_owned());
        let second = stripe.ensure_price(&s).await.unwrap();

        // Assert: no recreation, nickname updated in place.
        assert!(!second.created);
        assert_eq!(first.price_id, second.price_id);
        assert_eq!(stripe.all_prices().len(), 1);
        assert_eq!(stripe.active_prices()[0].nickname.as_deref(), Some("pico"));
    }

    #[tokio::test]
    async fn ensure_price_creates_new_and_retires_old_on_amount_change() {
        // Arrange
        let stripe = FakeStripeClient::default();
        let old = stripe.ensure_price(&spec("k-monthly", 2500)).await.unwrap();

        // Act
        let new = stripe.ensure_price(&spec("k-monthly", 3000)).await.unwrap();

        // Assert
        assert!(new.created);
        assert_ne!(old.price_id, new.price_id);

        let active = stripe.active_prices();
        assert_eq!(active.len(), 1, "only the new price stays active");
        assert_eq!(active[0].unit_amount_cents, 3000);
        assert_eq!(active[0].lookup_key, "k-monthly");

        let all = stripe.all_prices();
        assert_eq!(all.len(), 2);
        let old_price = all.iter().find(|p| p.id == old.price_id).unwrap();
        assert!(!old_price.active);
        assert_eq!(old_price.lookup_key, "");
    }

    #[tokio::test]
    async fn ensure_product_upserts_name_and_description() {
        // Arrange
        let stripe = FakeStripeClient::default();

        // Act
        stripe
            .ensure_product("gitlab-runner-standard", "Standard", Some("v1"))
            .await
            .unwrap();
        stripe
            .ensure_product("gitlab-runner-standard", "Standard Plus", Some("v2"))
            .await
            .unwrap();

        // Assert
        let products = stripe.products();
        assert_eq!(
            products.len(),
            1,
            "product id is stable, upserted not duplicated"
        );
        assert_eq!(products[0].1, "Standard Plus");
        assert_eq!(products[0].2.as_deref(), Some("v2"));
    }

    fn set(items: &[&str]) -> std::collections::HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn orphan_prices_flags_undeclared_and_keyless() {
        // Arrange
        let managed = vec![
            super::super::ManagedPrice {
                id: "price_kept".to_owned(),
                lookup_key: Some("declared-v1-monthly".to_owned()),
            },
            super::super::ManagedPrice {
                id: "price_orphan".to_owned(),
                lookup_key: Some("removed-v1-monthly".to_owned()),
            },
            super::super::ManagedPrice {
                id: "price_keyless".to_owned(),
                lookup_key: None,
            },
        ];
        let declared = set(&["declared-v1-monthly"]);

        // Act
        let orphans = super::orphan_prices(&managed, &declared);

        // Assert: the undeclared and the keyless prices are orphans; the
        // declared one is kept.
        let ids: Vec<&str> = orphans.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(ids, vec!["price_orphan", "price_keyless"]);
    }

    #[test]
    fn orphan_products_flags_undeclared() {
        // Arrange
        let managed = vec![
            super::super::ManagedProduct {
                id: "prod_kept".to_owned(),
            },
            super::super::ManagedProduct {
                id: "prod_orphan".to_owned(),
            },
        ];
        let declared = set(&["prod_kept"]);

        // Act
        let orphans = super::orphan_products(&managed, &declared);

        // Assert
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].id, "prod_orphan");
    }

    #[tokio::test]
    async fn fake_lists_managed_prices_and_archives() {
        // Arrange: two managed prices exist.
        let stripe = FakeStripeClient::default();
        stripe
            .ensure_price(&spec("a-v1-monthly", 1000))
            .await
            .unwrap();
        let b = stripe
            .ensure_price(&spec("b-v1-monthly", 2000))
            .await
            .unwrap();

        // Act: list, then archive one.
        let listed = stripe.list_managed_prices().await.unwrap();
        stripe.delete_or_archive_price(&b.price_id).await.unwrap();
        let after = stripe.list_managed_prices().await.unwrap();

        // Assert
        assert_eq!(listed.len(), 2);
        assert_eq!(after.len(), 1, "archived price no longer listed as active");
        assert_eq!(after[0].lookup_key.as_deref(), Some("a-v1-monthly"));
    }
}
