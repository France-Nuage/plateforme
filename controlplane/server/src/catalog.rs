//! Catalogue synchronization: reconciles `catalog.yaml` into Stripe and the
//! database. Runs at startup ([`sync_at_boot`]) and as the `catalog` CLI
//! subcommands for on-demand runs.

use std::path::{Path, PathBuf};

use frn_core::billing::Billing;
use frn_core::billing::stripe::HttpStripeClient;
use frn_core::managed::{Catalog, ManagedServices};

use crate::config::Config;
use crate::error::Error;

/// Catalogue path relative to the cwd, used by the `catalog` CLI subcommands.
pub const DEFAULT_CATALOG_PATH: &str = "catalog/catalog.yaml";

/// Absolute path where the release image ships the catalogue (see `Dockerfile`).
/// The served binary has no fixed working directory, so startup resolves here.
pub const BUNDLED_CATALOG_PATH: &str = "/app/catalog/catalog.yaml";

/// Validates the catalogue file without touching Stripe or the database, so CI
/// can fail fast on a malformed catalogue.
pub fn run_validate(catalog_path: &Path) -> Result<(), Error> {
    let catalog = Catalog::from_path(catalog_path)?;
    tracing::info!(
        managed_services = catalog.managed_services.len(),
        resources = catalog.resources.len(),
        legacy = catalog.legacy.len(),
        lookup_keys = catalog.all_lookup_keys().len(),
        "catalogue is valid"
    );
    Ok(())
}

/// Reconciles the catalogue into Stripe and the database. Requires
/// `STRIPE_SECRET_KEY`.
pub async fn run_sync(config: Config, catalog_path: &Path) -> Result<(), Error> {
    let stripe_key = config.stripe_secret_key.clone().ok_or_else(|| {
        Error::Config("STRIPE_SECRET_KEY is required for catalog sync".to_owned())
    })?;

    let catalog = Catalog::from_path(catalog_path)?;
    tracing::info!(
        managed_services = catalog.managed_services.len(),
        resources = catalog.resources.len(),
        legacy = catalog.legacy.len(),
        "loaded catalogue"
    );

    let managed = ManagedServices::new(
        config.app.auth.clone(),
        config.pool.clone(),
        config.managed_platform_config.clone(),
    );
    let billing = Billing::new(
        config.pool.clone(),
        HttpStripeClient::new(stripe_key),
        managed,
        config.kubeconfig_encryption_kek.clone(),
        // Checkout URLs are unused by sync; supply placeholders.
        String::new(),
        String::new(),
    );

    billing.sync_catalog(&catalog).await?;
    tracing::info!("catalogue synchronized into Stripe and database");
    Ok(())
}

/// Reconciles the catalogue into Stripe at startup, blocking so a ready instance
/// resolves the price ids a paid checkout needs. No-op without billing.
///
/// Version discovery is intentionally left out — see [`spawn_version_discovery`],
/// which is too slow to sit on the readiness path.
pub async fn sync_at_boot(config: &Config) -> Result<(), Error> {
    if config.stripe_secret_key.is_none() {
        tracing::info!("billing not configured, skipping Stripe catalogue sync at startup");
        return Ok(());
    }
    let catalog_path = boot_catalog_path();
    tracing::info!(path = %catalog_path.display(), "reconciling catalogue into Stripe at startup");
    run_sync(config.clone(), &catalog_path).await
}

/// Discovers deployable chart versions from the OCI registry in the background,
/// so the slow, external, best-effort walk never delays server readiness.
pub fn spawn_version_discovery(config: &Config) {
    let auth = config.app.auth.clone();
    let pool = config.pool.clone();
    let platform_config = config.managed_platform_config.clone();
    let credentials = config.charts_registry_credentials.clone();

    tokio::spawn(async move {
        let catalog = match Catalog::from_path(boot_catalog_path()) {
            Ok(catalog) => catalog,
            Err(error) => {
                tracing::warn!(%error, "cannot load catalogue for version discovery");
                return;
            }
        };
        let managed = ManagedServices::new(auth, pool, platform_config);
        managed
            .sync_versions_from_registry(&catalog, credentials.as_ref())
            .await;
        tracing::info!("chart version discovery finished");
    });
}

/// Resolves the startup catalogue path: `CATALOG_PATH` override, else the
/// bundled absolute path when present, else the relative default.
fn boot_catalog_path() -> PathBuf {
    if let Ok(path) = std::env::var("CATALOG_PATH") {
        return PathBuf::from(path);
    }
    let bundled = PathBuf::from(BUNDLED_CATALOG_PATH);
    if bundled.exists() {
        return bundled;
    }
    PathBuf::from(DEFAULT_CATALOG_PATH)
}

/// Archives catalogue-owned Stripe objects no longer declared. Dry-run unless
/// `force`; only objects tagged `managed_by=france-nuage-catalog` are considered.
pub async fn run_prune(config: Config, catalog_path: &Path, force: bool) -> Result<(), Error> {
    let stripe_key = config.stripe_secret_key.clone().ok_or_else(|| {
        Error::Config("STRIPE_SECRET_KEY is required for catalog prune".to_owned())
    })?;

    let catalog = Catalog::from_path(catalog_path)?;

    let managed = ManagedServices::new(
        config.app.auth.clone(),
        config.pool.clone(),
        config.managed_platform_config.clone(),
    );
    let billing = Billing::new(
        config.pool.clone(),
        HttpStripeClient::new(stripe_key),
        managed,
        config.kubeconfig_encryption_kek.clone(),
        String::new(),
        String::new(),
    );

    let report = billing.prune_catalog(&catalog, !force).await?;

    if report.is_empty() {
        tracing::info!("prune: nothing to archive, Stripe is aligned with the catalogue");
        return Ok(());
    }

    let mode = if report.archived {
        "archived"
    } else {
        "would archive (dry-run; pass --force to apply)"
    };
    tracing::warn!(
        prices = report.prices.len(),
        products = report.products.len(),
        "prune: {mode}"
    );
    for price in &report.prices {
        tracing::warn!(
            id = %price.id,
            lookup_key = price.lookup_key.as_deref().unwrap_or("(none)"),
            "orphan price"
        );
    }
    for product in &report.products {
        tracing::warn!(id = %product.id, "orphan product");
    }
    Ok(())
}
