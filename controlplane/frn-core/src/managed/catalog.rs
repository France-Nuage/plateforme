//! Declarative managed-services catalogue.
//!
//! The catalogue is France Nuage's single source of truth for everything billed
//! through Stripe. It is declared in a versioned `catalog.yaml` and reconciled
//! into Stripe (and, for deployable apps, the database): every entry generates
//! Stripe products and prices, and the Stripe `price_...` ids are produced by
//! reconciliation rather than declared here. Because the catalogue is
//! exhaustive, Stripe can be fully regenerated from it — identically in
//! production and in a test sandbox.
//!
//! The catalogue has three sections, all of which generate Stripe
//! products/prices but differ in whether they are deployable and displayed:
//!
//! - [`Catalog::managed_services`]: handpicked apps deployable via a Helm chart
//!   and shown in the console.
//! - [`Catalog::resources`]: bare Kubernetes resources billed by usage (vCPU,
//!   RAM, storage). No chart, not shown as an installable app.
//! - [`Catalog::legacy`]: historical VM "instance" products kept for
//!   not-yet-migrated customers. No chart, not shown; retired as a block once no
//!   customer remains.
//!
//! This module defines the schema (serde structs) and the parser only.
//! Reconciling the parsed catalogue into Stripe/DB is handled by the caller,
//! keeping parsing pure and unit-testable.

use serde::Deserialize;

use crate::billing::PriceInterval;
use crate::managed::{ManagedDatabaseEngine, ManagedServiceCategory};

/// Error raised while loading or parsing the catalogue.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("failed to read catalogue file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse catalogue yaml: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("invalid catalogue: {0}")]
    Invalid(String),
}

/// Root of the declarative catalogue.
///
/// All three sections are reconciled into Stripe. `managed_services` are
/// additionally reconciled into the database and displayed in the console.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// Deployable managed applications (product + Helm chart + plans).
    #[serde(default)]
    pub managed_services: Vec<ManagedServiceEntry>,
    /// Bare Kubernetes resources billed by usage (no chart, not displayed).
    #[serde(default)]
    pub resources: Vec<BillableProduct>,
    /// Historical VM instance products (no chart, not displayed, being retired).
    #[serde(default)]
    pub legacy: Vec<BillableProduct>,
}

/// A billing recurrence, mirroring [`PriceInterval`] in the catalogue schema.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogInterval {
    Month,
    Year,
}

impl From<CatalogInterval> for PriceInterval {
    fn from(interval: CatalogInterval) -> Self {
        match interval {
            CatalogInterval::Month => PriceInterval::Month,
            CatalogInterval::Year => PriceInterval::Year,
        }
    }
}

/// How a plan's price is computed, and whether a quantity (seats) is required.
///
/// This is France Nuage's product-level notion (declared on the plan), distinct
/// from the Stripe `billing_scheme` (declared on the price). It answers "must
/// the console ask for a seat count, and must checkout require one?".
///
/// - `flat` (default): a single fixed fee, billed quantity 1. No seats.
/// - `per_unit`: a fixed amount per seat, times the declared seat count.
/// - `tiered`: graduated/volume tiers over the declared seat count.
///
/// Every existing plan omits this field and therefore defaults to `flat`, so
/// the whole existing catalogue stays valid unchanged.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    #[default]
    Flat,
    PerUnit,
    Tiered,
}

impl PricingModel {
    /// Returns the snake_case string persisted in `managed.service_plan` and
    /// carried over the RPC/SDK (`flat` | `per_unit` | `tiered`).
    pub fn as_str(&self) -> &'static str {
        match self {
            PricingModel::Flat => "flat",
            PricingModel::PerUnit => "per_unit",
            PricingModel::Tiered => "tiered",
        }
    }

    /// Whether this pricing model requires a seat quantity at checkout.
    pub fn requires_seats(&self) -> bool {
        !matches!(self, PricingModel::Flat)
    }
}

/// Stripe `billing_scheme` on a price: the two values Stripe supports.
///
/// Defaults to `per_unit` (Stripe's own default), which is what every existing
/// "flat" price already is (billed with quantity 1). `tiered` selects the
/// `tiers` + `tiers_mode` model.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BillingScheme {
    #[default]
    PerUnit,
    Tiered,
}

/// Stripe `tiers_mode`, required on a `tiered` price.
///
/// `graduated` bills each tier's units at that tier's rate and sums them;
/// `volume` applies the single tier the total quantity lands in to all units.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TiersMode {
    Graduated,
    Volume,
}

/// The upper bound of a pricing tier: a finite seat count, or `inf` (fallback).
///
/// Declared in the YAML as either an integer (`up_to: 100`) or the literal
/// string `inf` (`up_to: inf`), matching Stripe's `tiers[i][up_to]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierUpTo {
    /// Inclusive upper bound (a positive seat count).
    Finite(i64),
    /// Unbounded fallback tier (Stripe `inf`); only valid as the last tier.
    Inf,
}

impl<'de> Deserialize<'de> for TierUpTo {
    /// Accepts a positive integer or the exact literal string `inf`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Int(i64),
            Str(String),
        }

        match Raw::deserialize(deserializer)? {
            Raw::Int(n) => Ok(TierUpTo::Finite(n)),
            Raw::Str(s) if s == "inf" => Ok(TierUpTo::Inf),
            Raw::Str(s) => Err(D::Error::custom(format!(
                "invalid tier up_to '{s}', expected a positive integer or 'inf'"
            ))),
        }
    }
}

/// A single pricing tier for a `tiered` price.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogTier {
    /// Inclusive upper bound of this tier (finite) or `inf` for the last tier.
    pub up_to: TierUpTo,
    /// Per-seat amount in the currency's smallest unit (cents). Kept as
    /// `unit_amount_cents` for internal consistency; mapped to Stripe's
    /// `tiers[i][unit_amount]` at reconciliation.
    pub unit_amount_cents: i64,
}

/// A single recurring price to reconcile into Stripe.
///
/// The `lookup_key` is declared explicitly (never generated): it is the stable
/// identity France Nuage owns for this price, aligned with what already exists
/// in Stripe so reconciliation reuses existing prices instead of duplicating
/// them. Amounts are in the currency's smallest unit (cents).
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CatalogPrice {
    /// Stable Stripe lookup key (e.g. `postgresql-managed-pico-v1-monthly`).
    pub lookup_key: String,
    /// Fixed amount in the currency's smallest unit (cents). Required for a
    /// `per_unit` price; omitted for a `tiered` price (amounts live in `tiers`).
    #[serde(default)]
    pub unit_amount_cents: Option<i64>,
    /// Three-letter ISO currency code, lowercase (e.g. `eur`).
    pub currency: String,
    /// Recurring billing interval. `None` for a one-time (one-shot) price, e.g.
    /// a fixed-fee service/prestation.
    #[serde(default)]
    pub interval: Option<CatalogInterval>,
    /// Optional Stripe nickname (internal label, hidden from customers).
    #[serde(default)]
    pub nickname: Option<String>,
    /// Stripe billing scheme. Optional, defaults to `per_unit`, so every
    /// existing price stays valid unchanged.
    #[serde(default)]
    pub billing_scheme: BillingScheme,
    /// Stripe `tiers_mode`, required when `billing_scheme` is `tiered`.
    #[serde(default)]
    pub tiers_mode: Option<TiersMode>,
    /// Pricing tiers, required (non-empty) when `billing_scheme` is `tiered`.
    #[serde(default)]
    pub tiers: Vec<CatalogTier>,
}

/// A Stripe product with prices but no chart/plan semantics.
///
/// Used for `resources` and `legacy`: these generate a Stripe product and its
/// prices for a faithful, regenerable catalogue, but are neither deployed via a
/// chart nor shown as installable services.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BillableProduct {
    /// Stable product slug (catalogue-local identifier).
    pub slug: String,
    /// Existing Stripe product id (`prod_...`) this entry maps to.
    ///
    /// Stripe product ids are arbitrary and immutable, and a price cannot be
    /// moved between products, so reconciliation targets the existing product
    /// rather than deriving an id from the slug. Required for entries whose
    /// product already exists in Stripe.
    pub stripe_product_id: String,
    /// Product display name.
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Prices offered for this product (each with an explicit lookup key).
    #[serde(default)]
    pub prices: Vec<CatalogPrice>,
}

/// A deployable managed application: product, Helm chart, and plans.
///
/// A managed service maps to a single Stripe product; its plans are Stripe
/// prices on that product (tiers/periods), mirroring how the products already
/// exist in production.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ManagedServiceEntry {
    pub slug: String,
    /// Existing Stripe product id (`prod_...`) this service maps to.
    pub stripe_product_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub category: ManagedServiceCategory,
    #[serde(default)]
    pub database_engine: Option<ManagedDatabaseEngine>,
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Label selector resolved at deployment (e.g. `{availability: fr}`).
    #[serde(default)]
    pub deploy_target: Option<serde_json::Value>,
    /// Helm chart backing this service.
    pub chart: CatalogChart,
    #[serde(default)]
    pub plans: Vec<CatalogPlan>,
}

/// Reference to the Helm chart that deploys a managed service.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CatalogChart {
    /// Chart name (matches the chart directory / OCI artifact name).
    pub name: String,
    /// OCI reference the chart is published to, when applicable.
    #[serde(default)]
    pub oci_reference: Option<String>,
}

/// A pricing tier for a managed service.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CatalogPlan {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Defaults to `active` when omitted.
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub highlighted: bool,
    /// Whether purchasing this plan requires payment. Defaults to `true`.
    #[serde(default = "default_true")]
    pub requires_payment: bool,
    /// Pricing model of the plan (flat / per_unit / tiered). Defaults to `flat`,
    /// so every existing plan stays valid unchanged. Drives whether a seat count
    /// is required at checkout and shown in the console. Must be coherent with
    /// the `billing_scheme` of the plan's prices (see [`Catalog::validate`]).
    #[serde(default)]
    pub pricing_model: PricingModel,
    /// Prices for this plan (each with an explicit lookup key). Empty for free
    /// plans (`requires_payment: false`).
    #[serde(default)]
    pub prices: Vec<CatalogPrice>,
    /// Helm values overrides for this plan (arbitrary JSON object).
    #[serde(default)]
    pub values_override: Option<serde_json::Value>,
    /// Plan entitlements (support level, SLA, etc.).
    #[serde(default)]
    pub entitlements: Vec<CatalogEntitlement>,
}

/// A single entitlement entry within a plan.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntitlement {
    pub key: String,
    pub label: String,
    pub value: String,
}

fn default_status() -> String {
    "active".to_owned()
}

fn default_true() -> bool {
    true
}

impl Catalog {
    /// Parses a catalogue from a YAML string and validates it.
    pub fn from_yaml(yaml: &str) -> Result<Self, CatalogError> {
        let catalog: Catalog = serde_yaml::from_str(yaml)?;
        catalog.validate()?;
        Ok(catalog)
    }

    /// Loads and parses a catalogue from a file path.
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, CatalogError> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_yaml(&contents)
    }

    /// Returns every lookup key declared across all sections.
    ///
    /// Used by pruning to determine which catalogue-owned Stripe prices are
    /// still declared (any active managed price whose lookup key is absent here
    /// is an orphan to retire).
    pub fn all_lookup_keys(&self) -> std::collections::HashSet<String> {
        let mut keys = std::collections::HashSet::new();
        for service in &self.managed_services {
            for plan in &service.plans {
                for price in &plan.prices {
                    keys.insert(price.lookup_key.clone());
                }
            }
        }
        for product in self.resources.iter().chain(&self.legacy) {
            for price in &product.prices {
                keys.insert(price.lookup_key.clone());
            }
        }
        keys
    }

    /// Returns every Stripe product id declared across all sections.
    pub fn all_stripe_product_ids(&self) -> std::collections::HashSet<String> {
        let mut ids = std::collections::HashSet::new();
        for service in &self.managed_services {
            ids.insert(service.stripe_product_id.clone());
        }
        for product in self.resources.iter().chain(&self.legacy) {
            ids.insert(product.stripe_product_id.clone());
        }
        ids
    }

    /// Validates invariants that serde alone cannot express.
    ///
    /// Ensures:
    /// - lookup keys are unique across the whole catalogue (a lookup key
    ///   identifies exactly one price in Stripe);
    /// - a payment-requiring plan with no prices is rejected;
    /// - every price is internally consistent with its `billing_scheme`
    ///   (`per_unit` needs an amount and forbids tiers; `tiered` needs a
    ///   `tiers_mode` and a tier list whose last entry is `inf`, and forbids a
    ///   top-level amount);
    /// - each plan's `pricing_model` is coherent with the `billing_scheme` of
    ///   its prices (`tiered` plan ⇒ tiered prices; `flat`/`per_unit` plan ⇒
    ///   `per_unit` prices).
    fn validate(&self) -> Result<(), CatalogError> {
        let mut seen = std::collections::HashSet::new();

        let mut check_price = |price: &CatalogPrice| -> Result<(), CatalogError> {
            if !seen.insert(price.lookup_key.clone()) {
                return Err(CatalogError::Invalid(format!(
                    "duplicate lookup_key '{}'",
                    price.lookup_key
                )));
            }
            validate_price_structure(price).map_err(CatalogError::Invalid)
        };

        for service in &self.managed_services {
            for plan in &service.plans {
                if plan.requires_payment && plan.prices.is_empty() {
                    return Err(CatalogError::Invalid(format!(
                        "plan '{}/{}' requires payment but declares no prices",
                        service.slug, plan.slug
                    )));
                }
                for price in &plan.prices {
                    check_price(price)?;
                    validate_plan_price_coherence(&service.slug, plan, price)
                        .map_err(CatalogError::Invalid)?;
                }
            }
        }
        for product in self.resources.iter().chain(&self.legacy) {
            for price in &product.prices {
                check_price(price)?;
            }
        }
        Ok(())
    }
}

/// Validates a single price's internal consistency with its `billing_scheme`.
///
/// Returns a human-readable error message (wrapped by the caller into
/// [`CatalogError::Invalid`]) when an invariant is violated.
fn validate_price_structure(price: &CatalogPrice) -> Result<(), String> {
    let key = &price.lookup_key;
    match price.billing_scheme {
        BillingScheme::PerUnit => {
            if price.unit_amount_cents.is_none() {
                return Err(format!(
                    "price '{key}': per_unit requires unit_amount_cents"
                ));
            }
            if !price.tiers.is_empty() || price.tiers_mode.is_some() {
                return Err(format!(
                    "price '{key}': per_unit must not declare tiers or tiers_mode"
                ));
            }
        }
        BillingScheme::Tiered => {
            if price.unit_amount_cents.is_some() {
                return Err(format!(
                    "price '{key}': tiered must not declare a top-level unit_amount_cents \
                     (amounts live in tiers)"
                ));
            }
            if price.tiers_mode.is_none() {
                return Err(format!("price '{key}': tiered requires tiers_mode"));
            }
            if price.tiers.is_empty() {
                return Err(format!("price '{key}': tiered requires at least one tier"));
            }
            let last = price.tiers.len() - 1;
            // Finite bounds must be strictly increasing positive integers, as
            // Stripe requires: each tier's `up_to` is an inclusive seat count
            // greater than the previous tier's.
            let mut prev_finite: Option<i64> = None;
            for (i, tier) in price.tiers.iter().enumerate() {
                let is_last = i == last;
                match (tier.up_to, is_last) {
                    (TierUpTo::Inf, true) => {}
                    (TierUpTo::Inf, false) => {
                        return Err(format!(
                            "price '{key}': only the last tier may use up_to: inf"
                        ));
                    }
                    (TierUpTo::Finite(_), true) => {
                        return Err(format!("price '{key}': the last tier must use up_to: inf"));
                    }
                    (TierUpTo::Finite(bound), false) => {
                        if bound <= 0 {
                            return Err(format!(
                                "price '{key}': tier up_to must be a positive integer, got {bound}"
                            ));
                        }
                        if let Some(prev) = prev_finite
                            && bound <= prev
                        {
                            return Err(format!(
                                "price '{key}': tier up_to values must be strictly \
                                 increasing (got {bound} after {prev})"
                            ));
                        }
                        prev_finite = Some(bound);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Validates that a plan's `pricing_model` matches its price's `billing_scheme`.
///
/// A `tiered` plan must carry `tiered` prices; a `flat` or `per_unit` plan must
/// carry `per_unit` prices (both are billed with Stripe `per_unit`, they differ
/// only in whether a seat quantity is requested).
fn validate_plan_price_coherence(
    service_slug: &str,
    plan: &CatalogPlan,
    price: &CatalogPrice,
) -> Result<(), String> {
    let expected = match plan.pricing_model {
        PricingModel::Tiered => BillingScheme::Tiered,
        PricingModel::Flat | PricingModel::PerUnit => BillingScheme::PerUnit,
    };
    if price.billing_scheme != expected {
        return Err(format!(
            "plan '{service_slug}/{}' has pricing_model {:?} but price '{}' has \
             billing_scheme {:?} (expected {:?})",
            plan.slug, plan.pricing_model, price.lookup_key, price.billing_scheme, expected
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
managed_services:
  - slug: gitlab-runner
    stripe_product_id: prod_gitlab
    name: GitLab Runner
    description: Runners Kubernetes managés.
    category: automation
    deploy_target:
      availability: fr
    chart:
      name: gitlab-runner
      oci_reference: oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/gitlab-runner
    plans:
      - slug: gitlab-runner-standard
        name: GitLab Runner
        highlighted: false
        requires_payment: true
        prices:
          - lookup_key: gitlab-managed-v1-monthly
            unit_amount_cents: 5000
            currency: eur
            interval: month
        entitlements:
          - key: buildkit
            label: Build
            value: BuildKit rootless

resources:
  - slug: k8s-vcpu
    stripe_product_id: prod_k8svcpu
    name: K8S - vCPU
    prices:
      - lookup_key: k8s-vcpu-v1-monthly
        unit_amount_cents: 1250
        currency: eur
        interval: month

legacy:
  - slug: instance-xs
    stripe_product_id: prod_instancexs
    name: Instance XS
    prices:
      - lookup_key: instance-xs-v1-monthly
        unit_amount_cents: 972
        currency: eur
        interval: month
"#;

    #[test]
    fn parses_the_three_sections() {
        // Arrange & Act
        let catalog = Catalog::from_yaml(SAMPLE).unwrap();

        // Assert
        assert_eq!(catalog.managed_services.len(), 1);
        assert_eq!(catalog.resources.len(), 1);
        assert_eq!(catalog.legacy.len(), 1);

        let service = &catalog.managed_services[0];
        assert_eq!(service.slug, "gitlab-runner");
        assert_eq!(service.category, ManagedServiceCategory::Automation);
        assert_eq!(service.chart.name, "gitlab-runner");

        let plan = &service.plans[0];
        assert_eq!(plan.pricing_model, PricingModel::Flat, "defaults to flat");
        let price = &plan.prices[0];
        assert_eq!(price.lookup_key, "gitlab-managed-v1-monthly");
        assert_eq!(price.unit_amount_cents, Some(5000));
        assert_eq!(price.interval, Some(CatalogInterval::Month));
        assert_eq!(
            price.billing_scheme,
            BillingScheme::PerUnit,
            "defaults to per_unit"
        );
    }

    #[test]
    fn rejects_duplicate_lookup_keys() {
        // Arrange: same lookup_key in a resource and a legacy product.
        let yaml = r#"
resources:
  - slug: a
    stripe_product_id: prod_a
    name: A
    prices:
      - { lookup_key: dup-v1-monthly, unit_amount_cents: 100, currency: eur, interval: month }
legacy:
  - slug: b
    stripe_product_id: prod_b
    name: B
    prices:
      - { lookup_key: dup-v1-monthly, unit_amount_cents: 200, currency: eur, interval: month }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    #[test]
    fn rejects_paid_plan_without_prices() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: svc
    stripe_product_id: prod_svc
    name: Svc
    category: automation
    chart: { name: svc }
    plans:
      - slug: paid
        name: Paid
        requires_payment: true
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    #[test]
    fn production_catalogue_file_is_valid() {
        // Arrange: the real catalogue shipped in controlplane/catalog.
        let yaml = include_str!("../../../catalog/catalog.yaml");

        // Act
        let catalog = Catalog::from_yaml(yaml).expect("catalog.yaml must be valid");

        // Assert: sections are populated as expected.
        assert!(!catalog.managed_services.is_empty());
        assert!(!catalog.resources.is_empty());
        assert!(!catalog.legacy.is_empty());
    }

    #[test]
    fn allows_free_plan_without_prices() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: svc
    stripe_product_id: prod_svc
    name: Svc
    category: automation
    chart: { name: svc }
    plans:
      - slug: free
        name: Free
        requires_payment: false
"#;

        // Act
        let catalog = Catalog::from_yaml(yaml).unwrap();

        // Assert
        let plan = &catalog.managed_services[0].plans[0];
        assert_eq!(plan.status, "active");
        assert!(!plan.requires_payment);
        assert!(plan.prices.is_empty());
    }

    /// A `per_unit` (per-seat, fixed amount) plan parses with the right scheme.
    #[test]
    fn parses_per_unit_seat_plan() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: caldiy
    stripe_product_id: prod_caldiy
    name: Cal.diy
    category: collaboration
    chart: { name: caldiy }
    plans:
      - slug: caldiy-team
        name: Cal.diy Team
        requires_payment: true
        pricing_model: per_unit
        prices:
          - lookup_key: caldiy-seat-v1-monthly
            billing_scheme: per_unit
            unit_amount_cents: 9000
            currency: eur
            interval: month
"#;

        // Act
        let catalog = Catalog::from_yaml(yaml).unwrap();

        // Assert
        let plan = &catalog.managed_services[0].plans[0];
        assert_eq!(plan.pricing_model, PricingModel::PerUnit);
        let price = &plan.prices[0];
        assert_eq!(price.billing_scheme, BillingScheme::PerUnit);
        assert_eq!(price.unit_amount_cents, Some(9000));
        assert!(price.tiers.is_empty());
    }

    /// A `tiered`/`graduated` plan parses its tiers, including `up_to: inf`.
    #[test]
    fn parses_tiered_graduated_plan() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: gitlab
    stripe_product_id: prod_gitlab
    name: GitLab CE
    category: automation
    chart: { name: gitlab }
    plans:
      - slug: gitlab-ce-graduated
        name: GitLab CE
        requires_payment: true
        pricing_model: tiered
        prices:
          - lookup_key: gitlab-ce-seat-graduated-v1-monthly
            billing_scheme: tiered
            tiers_mode: graduated
            currency: eur
            interval: month
            tiers:
              - { up_to: 100, unit_amount_cents: 1000 }
              - { up_to: 200, unit_amount_cents: 800 }
              - { up_to: inf, unit_amount_cents: 600 }
"#;

        // Act
        let catalog = Catalog::from_yaml(yaml).unwrap();

        // Assert
        let price = &catalog.managed_services[0].plans[0].prices[0];
        assert_eq!(price.billing_scheme, BillingScheme::Tiered);
        assert_eq!(price.tiers_mode, Some(TiersMode::Graduated));
        assert_eq!(price.unit_amount_cents, None);
        assert_eq!(price.tiers.len(), 3);
        assert_eq!(price.tiers[0].up_to, TierUpTo::Finite(100));
        assert_eq!(price.tiers[0].unit_amount_cents, 1000);
        assert_eq!(price.tiers[2].up_to, TierUpTo::Inf);
    }

    /// A `tiered`/`volume` plan parses with the volume tiers_mode.
    #[test]
    fn parses_tiered_volume_plan() {
        // Arrange
        let yaml = tiered_yaml("volume");

        // Act
        let catalog = Catalog::from_yaml(&yaml).unwrap();

        // Assert
        let price = &catalog.managed_services[0].plans[0].prices[0];
        assert_eq!(price.tiers_mode, Some(TiersMode::Volume));
    }

    /// A `tiered` price without `tiers_mode` is rejected.
    #[test]
    fn rejects_tiered_without_tiers_mode() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: tiered
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: tiered
            currency: eur
            interval: month
            tiers:
              - { up_to: inf, unit_amount_cents: 600 }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// A `tiered` price whose last tier is finite (no `inf`) is rejected.
    #[test]
    fn rejects_tiered_without_inf_last_tier() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: tiered
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: tiered
            tiers_mode: graduated
            currency: eur
            interval: month
            tiers:
              - { up_to: 100, unit_amount_cents: 1000 }
              - { up_to: 200, unit_amount_cents: 800 }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// A `tiered` price whose finite `up_to` bounds are not strictly increasing
    /// is rejected (Stripe requires ascending tier bounds).
    #[test]
    fn rejects_tiered_with_non_increasing_bounds() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: tiered
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: tiered
            tiers_mode: graduated
            currency: eur
            interval: month
            tiers:
              - { up_to: 100, unit_amount_cents: 1000 }
              - { up_to: 100, unit_amount_cents: 800 }
              - { up_to: inf, unit_amount_cents: 600 }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// A `tiered` price with a top-level `unit_amount_cents` is rejected.
    #[test]
    fn rejects_tiered_with_top_level_amount() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: tiered
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: tiered
            tiers_mode: graduated
            unit_amount_cents: 500
            currency: eur
            interval: month
            tiers:
              - { up_to: inf, unit_amount_cents: 600 }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// A `per_unit` price that declares tiers is rejected.
    #[test]
    fn rejects_per_unit_with_tiers() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: per_unit
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: per_unit
            unit_amount_cents: 9000
            currency: eur
            interval: month
            tiers:
              - { up_to: inf, unit_amount_cents: 600 }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// A `per_unit` price without an amount is rejected.
    #[test]
    fn rejects_per_unit_without_amount() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: per_unit
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: per_unit
            currency: eur
            interval: month
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// A plan whose `pricing_model` disagrees with its price `billing_scheme`
    /// is rejected (here: a `per_unit` plan carrying a `tiered` price).
    #[test]
    fn rejects_plan_price_scheme_mismatch() {
        // Arrange
        let yaml = r#"
managed_services:
  - slug: s
    stripe_product_id: prod_s
    name: S
    category: automation
    chart: { name: s }
    plans:
      - slug: p
        name: P
        requires_payment: true
        pricing_model: per_unit
        prices:
          - lookup_key: p-seat-v1-monthly
            billing_scheme: tiered
            tiers_mode: graduated
            currency: eur
            interval: month
            tiers:
              - { up_to: inf, unit_amount_cents: 600 }
"#;

        // Act
        let result = Catalog::from_yaml(yaml);

        // Assert
        assert!(matches!(result, Err(CatalogError::Invalid(_))));
    }

    /// Builds a valid tiered plan YAML for the given tiers_mode.
    fn tiered_yaml(mode: &str) -> String {
        format!(
            r#"
managed_services:
  - slug: gitlab
    stripe_product_id: prod_gitlab
    name: GitLab CE
    category: automation
    chart: {{ name: gitlab }}
    plans:
      - slug: gitlab-ce
        name: GitLab CE
        requires_payment: true
        pricing_model: tiered
        prices:
          - lookup_key: gitlab-ce-seat-v1-monthly
            billing_scheme: tiered
            tiers_mode: {mode}
            currency: eur
            interval: month
            tiers:
              - {{ up_to: 100, unit_amount_cents: 1000 }}
              - {{ up_to: inf, unit_amount_cents: 600 }}
"#
        )
    }
}
