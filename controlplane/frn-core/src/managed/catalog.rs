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
    /// Amount in the currency's smallest unit (cents).
    pub unit_amount_cents: i64,
    /// Three-letter ISO currency code, lowercase (e.g. `eur`).
    pub currency: String,
    /// Recurring billing interval. `None` for a one-time (one-shot) price, e.g.
    /// a fixed-fee service/prestation.
    #[serde(default)]
    pub interval: Option<CatalogInterval>,
    /// Optional Stripe nickname (internal label, hidden from customers).
    #[serde(default)]
    pub nickname: Option<String>,
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
    /// Ensures lookup keys are unique across the whole catalogue (a lookup key
    /// identifies exactly one price in Stripe) and that a payment-requiring plan
    /// with no prices is rejected.
    fn validate(&self) -> Result<(), CatalogError> {
        let mut seen = std::collections::HashSet::new();

        let mut check_price = |price: &CatalogPrice| -> Result<(), CatalogError> {
            if !seen.insert(price.lookup_key.clone()) {
                return Err(CatalogError::Invalid(format!(
                    "duplicate lookup_key '{}'",
                    price.lookup_key
                )));
            }
            Ok(())
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

        let price = &service.plans[0].prices[0];
        assert_eq!(price.lookup_key, "gitlab-managed-v1-monthly");
        assert_eq!(price.unit_amount_cents, 5000);
        assert_eq!(price.interval, Some(CatalogInterval::Month));
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
}
