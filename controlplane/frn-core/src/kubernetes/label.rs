//! Cluster label registry and admin CRUD.
//!
//! Labels are key/value pairs (e.g. `availability=ft`) attached to Kubernetes
//! clusters by platform admins. Managed services declare the labels they
//! require through `managed.service.deploy_target`; at instance deployment the
//! control plane picks a healthy cluster carrying ALL the required pairs (see
//! [`KubernetesClusters::pick_healthy_cluster_matching`]). Labels flagged
//! `system` are owned by the control plane: the API refuses to create, delete,
//! attach or detach them, even for platform admins.
//!
//! [`KubernetesClusters::pick_healthy_cluster_matching`]: crate::kubernetes::KubernetesClusters::pick_healthy_cluster_matching

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use fabrique::{Delete, Factory, Model, Query};
use serde::Serialize;
use sqlx::{FromRow, Pool, Postgres};
use thiserror::Error;
use uuid::Uuid;

use crate::authorization::Principal;
use crate::kubernetes::{KubernetesCluster, KubernetesClusterIdColumn};

/// Maximum length of a label key or value, matching the `length(...) < 50`
/// database CHECK.
const MAX_LABEL_PART_LENGTH: usize = 49;

/// A cluster label row: a reusable key/value pair.
///
/// `key` and `value` are CITEXT in the database, so lookups and the
/// deploy-target matching are case-insensitive.
#[derive(Debug, Clone, Factory, Model, Serialize)]
#[fabrique(table = "kubernetes.label")]
pub struct KubernetesLabel {
    #[fabrique(primary_key)]
    pub id: Uuid,
    pub key: String,
    pub value: String,
    /// `true` when the label is owned by the control plane (internal code or
    /// seed). Such labels are read-only through the API.
    pub system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Minimal reference to a managed service whose `deploy_target` requires a
/// label. Returned by [`KubernetesLabels::list_services_referencing_label`]
/// so operators can see which services lose deployment eligibility before
/// detaching or deleting the label.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ManagedServiceRef {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

/// Attachment of a label to a cluster (join row).
#[derive(Debug, Clone, Model, Serialize)]
#[fabrique(table = "kubernetes.cluster_label")]
pub struct KubernetesClusterLabel {
    #[fabrique(primary_key)]
    pub id: Uuid,
    #[fabrique(belongs_to = KubernetesCluster)]
    pub cluster_id: Uuid,
    #[fabrique(belongs_to = KubernetesLabel)]
    pub label_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Errors raised by the label service layer.
#[derive(Debug, Error)]
pub enum KubernetesLabelError {
    #[error("forbidden")]
    Forbidden,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("fabrique error: {0}")]
    Fabrique(#[from] fabrique::Error),
    #[error("label not found: {0}")]
    NotFound(Uuid),
    #[error("cluster not found: {0}")]
    ClusterNotFound(Uuid),
    #[error("label already exists: {key}={value}")]
    AlreadyExists { key: String, value: String },
    #[error("invalid label key: {0}")]
    InvalidKey(String),
    #[error("invalid label value: {0}")]
    InvalidValue(String),
    #[error("label {0} is managed by the control plane and is read-only")]
    SystemLabelReadOnly(Uuid),
}

/// Service layer for the platform-admin cluster label registry.
///
/// All operations are restricted to platform administrators
/// (`Principal::is_platform_admin`). System labels (`system = true`) are
/// rejected on every mutating operation: they belong to the control plane.
#[derive(Clone)]
pub struct KubernetesLabels {
    db: Pool<Postgres>,
}

impl KubernetesLabels {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    /// Lists every label, ordered by key then value.
    pub async fn list_labels<P: Principal + Sync>(
        &self,
        principal: &P,
    ) -> Result<Vec<KubernetesLabel>, KubernetesLabelError> {
        require_admin(principal)?;
        let mut labels = KubernetesLabel::query().select().get(&self.db).await?;
        sort_labels(&mut labels);
        Ok(labels)
    }

    /// Creates a user-managed label (`system = false`).
    pub async fn create_label<P: Principal + Sync>(
        &self,
        principal: &P,
        key: String,
        value: String,
    ) -> Result<KubernetesLabel, KubernetesLabelError> {
        require_admin(principal)?;
        validate_part(&key).map_err(KubernetesLabelError::InvalidKey)?;
        validate_part(&value).map_err(KubernetesLabelError::InvalidValue)?;

        if self.find_by_key_value(&key, &value).await?.is_some() {
            return Err(KubernetesLabelError::AlreadyExists { key, value });
        }

        KubernetesLabel::query()
            .insert()
            .set(KubernetesLabel::ID, Uuid::new_v4())
            .set(KubernetesLabel::KEY, key)
            .set(KubernetesLabel::VALUE, value)
            .set(KubernetesLabel::SYSTEM, false)
            .returning()
            .first(&self.db)
            .await?
            .ok_or(KubernetesLabelError::Database(sqlx::Error::RowNotFound))
    }

    /// Deletes a user-managed label. Attachments are removed by the
    /// `ON DELETE CASCADE` on `kubernetes.cluster_label`.
    pub async fn delete_label<P: Principal + Sync>(
        &self,
        principal: &P,
        label_id: Uuid,
    ) -> Result<(), KubernetesLabelError> {
        require_admin(principal)?;
        let label = self.find_user_label(label_id).await?;

        label.delete(&self.db).await?;
        Ok(())
    }

    /// Attaches a user-managed label to a cluster. Idempotent: attaching an
    /// already-attached label succeeds without effect.
    pub async fn attach_label<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
        label_id: Uuid,
    ) -> Result<(), KubernetesLabelError> {
        require_admin(principal)?;
        self.find_user_label(label_id).await?;
        self.ensure_cluster_exists(cluster_id).await?;

        // Raw SQL: ON CONFLICT targets the non-PK UNIQUE(cluster_id, label_id)
        // constraint, which the query builder cannot express.
        sqlx::query(
            "INSERT INTO kubernetes.cluster_label (id, cluster_id, label_id)
             VALUES ($1, $2, $3)
             ON CONFLICT (cluster_id, label_id) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(cluster_id)
        .bind(label_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Detaches a user-managed label from a cluster. Idempotent: detaching a
    /// label that is not attached succeeds without effect.
    pub async fn detach_label<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
        label_id: Uuid,
    ) -> Result<(), KubernetesLabelError> {
        require_admin(principal)?;
        self.find_user_label(label_id).await?;

        if let Some(attachment) = self.find_attachment(cluster_id, label_id).await? {
            attachment.delete(&self.db).await?;
        }
        Ok(())
    }

    /// Validates that every id maps to an existing user-managed label.
    ///
    /// Used to fail fast before an expensive operation (e.g. the cluster
    /// reachability check at creation) when a label id is unknown or points
    /// to a system label.
    pub async fn ensure_user_labels_exist<P: Principal + Sync>(
        &self,
        principal: &P,
        label_ids: &[Uuid],
    ) -> Result<(), KubernetesLabelError> {
        require_admin(principal)?;
        for label_id in label_ids {
            self.find_user_label(*label_id).await?;
        }
        Ok(())
    }

    /// Attaches several user-managed labels to a cluster, validating all of
    /// them before attaching any. Each attachment is idempotent, like
    /// [`Self::attach_label`].
    pub async fn attach_labels<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
        label_ids: &[Uuid],
    ) -> Result<(), KubernetesLabelError> {
        self.ensure_user_labels_exist(principal, label_ids).await?;
        for label_id in label_ids {
            self.attach_label(principal, cluster_id, *label_id).await?;
        }
        Ok(())
    }

    /// Lists the labels attached to a single cluster, ordered by key then
    /// value.
    pub async fn list_cluster_labels<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
    ) -> Result<Vec<KubernetesLabel>, KubernetesLabelError> {
        require_admin(principal)?;
        let mut labels = KubernetesLabel::query()
            .join::<KubernetesClusterLabel>()
            .select()
            .r#where(KubernetesClusterLabel::CLUSTER_ID, "=", cluster_id)
            .get(&self.db)
            .await?;
        sort_labels(&mut labels);
        Ok(labels)
    }

    /// Lists every (cluster id, label) attachment, so callers can hydrate the
    /// labels of many clusters without a per-cluster round-trip. Two queries
    /// regardless of the cluster count: the query builder returns the columns
    /// of a single model, so attachments and labels are fetched separately and
    /// paired in memory.
    pub async fn list_all_cluster_labels<P: Principal + Sync>(
        &self,
        principal: &P,
    ) -> Result<Vec<(Uuid, KubernetesLabel)>, KubernetesLabelError> {
        require_admin(principal)?;
        let attachments = KubernetesClusterLabel::query()
            .select()
            .get(&self.db)
            .await?;
        let labels: Vec<KubernetesLabel> = KubernetesLabel::query().select().get(&self.db).await?;
        let labels_by_id: HashMap<Uuid, KubernetesLabel> =
            labels.into_iter().map(|label| (label.id, label)).collect();

        let mut pairs: Vec<(Uuid, KubernetesLabel)> = attachments
            .into_iter()
            .filter_map(|attachment: KubernetesClusterLabel| {
                labels_by_id
                    .get(&attachment.label_id)
                    .map(|label| (attachment.cluster_id, label.clone()))
            })
            .collect();
        pairs.sort_by_key(|(_, label)| label_sort_key(label));
        Ok(pairs)
    }

    /// Lists the active managed services whose `deploy_target` requires the
    /// label's key/value pair, ordered by name.
    ///
    /// Read-only guard used by the console before detaching or deleting a
    /// label: removing it never breaks running instances (placement happens
    /// at deployment only), but the listed services may lose every eligible
    /// cluster for future deployments. Works on any label, including system
    /// ones.
    pub async fn list_services_referencing_label<P: Principal + Sync>(
        &self,
        principal: &P,
        label_id: Uuid,
    ) -> Result<Vec<ManagedServiceRef>, KubernetesLabelError> {
        require_admin(principal)?;
        let label = self.find_label(label_id).await?;

        // Raw SQL: the query builder cannot express jsonb_each_text or the
        // ::citext parameter casts. A service references the label when its
        // deploy_target carries the pair; ::citext keeps the comparison
        // case-insensitive, mirroring pick_healthy_cluster_matching. A NULL
        // deploy_target yields no rows from jsonb_each_text, so undeployable
        // services are naturally excluded.
        sqlx::query_as::<_, ManagedServiceRef>(
            "SELECT s.id, s.slug, s.name
             FROM managed.service s
             WHERE s.deactivated_at IS NULL
               AND EXISTS (
                   SELECT 1
                   FROM jsonb_each_text(s.deploy_target) AS target(key, value)
                   WHERE target.key::citext = $1::citext
                     AND target.value::citext = $2::citext
               )
             ORDER BY s.name",
        )
        .bind(&label.key)
        .bind(&label.value)
        .fetch_all(&self.db)
        .await
        .map_err(Into::into)
    }

    async fn find_label(&self, label_id: Uuid) -> Result<KubernetesLabel, KubernetesLabelError> {
        KubernetesLabel::query()
            .select()
            .r#where(KubernetesLabel::ID, "=", label_id)
            .first(&self.db)
            .await?
            .ok_or(KubernetesLabelError::NotFound(label_id))
    }

    async fn find_user_label(
        &self,
        label_id: Uuid,
    ) -> Result<KubernetesLabel, KubernetesLabelError> {
        let label = self.find_label(label_id).await?;
        if label.system {
            return Err(KubernetesLabelError::SystemLabelReadOnly(label_id));
        }
        Ok(label)
    }

    async fn find_attachment(
        &self,
        cluster_id: Uuid,
        label_id: Uuid,
    ) -> Result<Option<KubernetesClusterLabel>, KubernetesLabelError> {
        KubernetesClusterLabel::query()
            .select()
            .r#where(KubernetesClusterLabel::CLUSTER_ID, "=", cluster_id)
            .r#where(KubernetesClusterLabel::LABEL_ID, "=", label_id)
            .first(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn find_by_key_value(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Option<KubernetesLabel>, KubernetesLabelError> {
        // Raw SQL: the query builder cannot cast bind parameters. ::citext
        // keeps the lookup case-insensitive: without the cast Postgres
        // resolves citext = text to the case-sensitive text = text operator
        // and AVAILABILITY=FT would not be seen as a duplicate of
        // availability=ft.
        sqlx::query_as::<_, KubernetesLabel>(
            "SELECT * FROM kubernetes.label
             WHERE key = $1::citext AND value = $2::citext",
        )
        .bind(key)
        .bind(value)
        .fetch_optional(&self.db)
        .await
        .map_err(Into::into)
    }

    async fn ensure_cluster_exists(&self, cluster_id: Uuid) -> Result<(), KubernetesLabelError> {
        KubernetesCluster::query()
            .select()
            .r#where(KubernetesCluster::ID, "=", cluster_id)
            .first(&self.db)
            .await?
            .map(|_| ())
            .ok_or(KubernetesLabelError::ClusterNotFound(cluster_id))
    }
}

/// In-memory equivalent of `ORDER BY key, value` on the CITEXT columns: the
/// query builder accepts a single `order_by` call, so the two-column,
/// case-insensitive ordering is applied after fetching.
fn label_sort_key(label: &KubernetesLabel) -> (String, String) {
    (label.key.to_lowercase(), label.value.to_lowercase())
}

/// Sorts labels by key then value, ignoring case (see [`label_sort_key`]).
fn sort_labels(labels: &mut [KubernetesLabel]) {
    labels.sort_by_key(label_sort_key);
}

/// Validates a label key or value against the database CHECK constraint
/// (`length < 50`, charset `[a-zA-Z0-9-]`), so callers get a precise error
/// instead of an opaque constraint violation.
fn validate_part(part: &str) -> Result<(), String> {
    if part.is_empty() || part.len() > MAX_LABEL_PART_LENGTH {
        return Err(part.to_owned());
    }
    let bytes = part.as_bytes();
    if !bytes[0].is_ascii_alphanumeric() || !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
        return Err(part.to_owned());
    }
    if !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err(part.to_owned());
    }
    Ok(())
}

/// Rejects any principal that is not a platform administrator.
fn require_admin<P: Principal>(principal: &P) -> Result<(), KubernetesLabelError> {
    if principal.is_platform_admin() {
        Ok(())
    } else {
        Err(KubernetesLabelError::Forbidden)
    }
}
