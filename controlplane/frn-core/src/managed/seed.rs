//! Seed loader for the managed services catalog.
//!
//! Reads YAML files under a directory (one per service) and upserts the
//! corresponding `managed.service` rows and their plans. Versions are NOT
//! seeded in production: they are registered by the charts CI through the
//! `RegisterVersion` gRPC. A YAML file may declare a `dev_mock_version`
//! block for local development before the pipeline is wired up; the seed
//! only creates it when explicitly asked.
//!
//! Layout of each YAML file:
//!
//! ```yaml
//! service:
//!   slug: vaultwarden
//!   name: Vaultwarden
//!   category: security
//!   database_engine: cnpg   # optional
//!   description: ...        # optional
//!   icon_url: null          # optional
//! plans:                    # optional, always upserted
//!   - id: vaultwarden-standard
//!     name: Standard
//!     entitlements:
//!       - key: support_level
//!         label: Support
//!         value: Email
//!     prices:
//!       monthly: 999
//!       yearly: 10789
//! dev_mock_version:         # optional, dev-only
//!   chart_version: 0.0.1-mock
//!   app_version: 1.35.4     # optional
//!   oci_reference: oci://...
//!   configurable_values_schema: {...}  # optional JSON Schema
//!   ui_schema: {...}                   # optional rjsf UI Schema
//! ```

use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
use sqlx::{Pool, Postgres};
use thiserror::Error;
use uuid::Uuid;

use crate::managed::{ManagedDatabaseEngine, ManagedServiceCategory};

#[derive(Debug, Error)]
pub enum SeedError {
    #[error("io error reading seed directory: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not parse YAML file {path}: {source}")]
    Yaml {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Deserialize)]
struct SeedFile {
    service: ServiceSeed,
    #[serde(default)]
    plans: Vec<PlanSeed>,
    #[serde(default)]
    dev_mock_version: Option<DevMockVersionSeed>,
}

#[derive(Debug, Deserialize)]
struct ServiceSeed {
    slug: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    category: ManagedServiceCategory,
    #[serde(default)]
    database_engine: Option<ManagedDatabaseEngine>,
    #[serde(default)]
    icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlanSeed {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_active")]
    status: String,
    #[serde(default)]
    highlighted: bool,
    #[serde(default)]
    values: Option<Value>,
    #[serde(default)]
    entitlements: Vec<EntitlementSeed>,
    #[serde(default)]
    prices: Option<PricesSeed>,
}

fn default_active() -> String {
    "active".to_owned()
}

#[derive(Debug, Deserialize, Serialize)]
struct EntitlementSeed {
    key: String,
    label: String,
    value: String,
}

use serde::Serialize;

#[derive(Debug, Deserialize)]
struct PricesSeed {
    #[serde(default)]
    monthly: Option<i64>,
    #[serde(default)]
    yearly: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DevMockVersionSeed {
    chart_version: String,
    #[serde(default)]
    app_version: Option<String>,
    oci_reference: String,
    #[serde(default)]
    configurable_values_schema: Option<Value>,
    #[serde(default)]
    ui_schema: Option<Value>,
}

/// Reports applied to a single seed file. Returned for human-readable output
/// by the seed binary.
#[derive(Debug, Default)]
pub struct SeedReport {
    pub service_slug: String,
    pub plans_upserted: usize,
    pub mock_version_inserted: bool,
}

/// Reads every `*.yaml` / `*.yml` file under `dir` and applies the seed.
/// Set `with_dev_mock` to also insert the `dev_mock_version` block when
/// present (silently skipped if missing).
pub async fn seed_directory(
    pool: &Pool<Postgres>,
    dir: &Path,
    with_dev_mock: bool,
) -> Result<Vec<SeedReport>, SeedError> {
    let mut reports = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "yaml" | "yml") {
            continue;
        }
        let report = seed_file(pool, &path, with_dev_mock).await?;
        reports.push(report);
    }
    reports.sort_by(|a, b| a.service_slug.cmp(&b.service_slug));
    Ok(reports)
}

async fn seed_file(
    pool: &Pool<Postgres>,
    path: &Path,
    with_dev_mock: bool,
) -> Result<SeedReport, SeedError> {
    let raw = fs::read_to_string(path)?;
    let file: SeedFile = serde_yaml::from_str(&raw).map_err(|source| SeedError::Yaml {
        path: path.display().to_string(),
        source,
    })?;

    let service_id = upsert_service(pool, &file.service).await?;

    let mut plans_upserted = 0;
    for plan in &file.plans {
        upsert_plan(pool, service_id, plan).await?;
        plans_upserted += 1;
    }

    let mut mock_version_inserted = false;
    if with_dev_mock && let Some(mock) = &file.dev_mock_version {
        mock_version_inserted = insert_dev_mock_version(pool, service_id, mock).await?;
    }

    Ok(SeedReport {
        service_slug: file.service.slug,
        plans_upserted,
        mock_version_inserted,
    })
}

async fn upsert_service(pool: &Pool<Postgres>, service: &ServiceSeed) -> Result<Uuid, SeedError> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        r#"INSERT INTO managed.service
               (id, slug, name, description, category, database_engine, icon_url)
           VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6)
           ON CONFLICT (slug) DO UPDATE SET
               name = EXCLUDED.name,
               description = EXCLUDED.description,
               category = EXCLUDED.category,
               database_engine = EXCLUDED.database_engine,
               icon_url = EXCLUDED.icon_url,
               deactivated_at = NULL
           RETURNING id"#,
    )
    .bind(&service.slug)
    .bind(&service.name)
    .bind(&service.description)
    .bind(&service.category)
    .bind(&service.database_engine)
    .bind(&service.icon_url)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

async fn upsert_plan(
    pool: &Pool<Postgres>,
    service_id: Uuid,
    plan: &PlanSeed,
) -> Result<(), SeedError> {
    let entitlements = serde_json::to_value(&plan.entitlements)?;
    sqlx::query(
        r#"INSERT INTO managed.service_plan
               (id, service_id, slug, name, description, status, highlighted,
                values_override, entitlements, price_monthly_cents, price_yearly_cents)
           VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           ON CONFLICT (service_id, slug) DO UPDATE SET
               name = EXCLUDED.name,
               description = EXCLUDED.description,
               status = EXCLUDED.status,
               highlighted = EXCLUDED.highlighted,
               values_override = EXCLUDED.values_override,
               entitlements = EXCLUDED.entitlements,
               price_monthly_cents = EXCLUDED.price_monthly_cents,
               price_yearly_cents = EXCLUDED.price_yearly_cents"#,
    )
    .bind(service_id)
    .bind(&plan.id)
    .bind(&plan.name)
    .bind(&plan.description)
    .bind(&plan.status)
    .bind(plan.highlighted)
    .bind(&plan.values)
    .bind(&entitlements)
    .bind(plan.prices.as_ref().and_then(|p| p.monthly))
    .bind(plan.prices.as_ref().and_then(|p| p.yearly))
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_dev_mock_version(
    pool: &Pool<Postgres>,
    service_id: Uuid,
    mock: &DevMockVersionSeed,
) -> Result<bool, SeedError> {
    let inserted = sqlx::query(
        r#"INSERT INTO managed.service_version
               (id, service_id, chart_version, app_version, oci_reference,
                configurable_values_schema, ui_schema)
           VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6)
           ON CONFLICT (service_id, chart_version) DO UPDATE SET
               app_version = EXCLUDED.app_version,
               oci_reference = EXCLUDED.oci_reference,
               configurable_values_schema = EXCLUDED.configurable_values_schema,
               ui_schema = EXCLUDED.ui_schema"#,
    )
    .bind(service_id)
    .bind(&mock.chart_version)
    .bind(&mock.app_version)
    .bind(&mock.oci_reference)
    .bind(&mock.configurable_values_schema)
    .bind(&mock.ui_schema)
    .execute(pool)
    .await?;
    Ok(inserted.rows_affected() > 0)
}
