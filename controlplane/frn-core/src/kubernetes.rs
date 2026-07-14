//! Admin-managed Kubernetes cluster registry.
//!
//! Provides the entity and service layer for the platform-admin CRUD over
//! Kubernetes clusters. Each cluster stores an envelope-encrypted kubeconfig
//! (see the `frn-crypto` crate); before a cluster is created or its kubeconfig
//! changed, the control plane performs a synchronous reachability check against
//! the API server. The decrypted kubeconfig is exposed only through
//! [`KubernetesClusters::decrypt_kubeconfig`], which the deployment worker will
//! consume in a later iteration to deploy managed services onto these clusters.

mod label;

pub use label::*;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use fabrique::sql::operators::Direction;
use fabrique::{Model, Query};
use frn_crypto::{CURRENT_KEY_VERSION, EnvelopeCiphertext, Kek};
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client, Config};
use semver::Version;
use serde::Serialize;
use sqlx::{PgConnection, Pool, Postgres};
use strum_macros::{Display, EnumString};
use thiserror::Error;
use tokio::time::timeout;
use uuid::Uuid;

use crate::authorization::Principal;
use crate::managed::{ManagedServiceInstanceStatus, ManagedServiceInstanceView};

/// Maximum length of a cluster name, matching the DNS-label database CHECK.
const MAX_NAME_LENGTH: usize = 63;

/// Upper bound on the synchronous reachability check. The connect and read
/// timeouts are tighter; this guards the whole `GET /version` round-trip.
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Result of the most recent reachability check, mirrored from the database
/// `kubernetes_cluster_health_status` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "kubernetes_cluster_health_status",
    rename_all = "snake_case"
)]
pub enum KubernetesClusterHealthStatus {
    Healthy,
    Unreachable,
}

/// A registered Kubernetes cluster row.
///
/// Holds the envelope-encrypted kubeconfig material; the ciphertext is not
/// sensitive as long as the KEK stays out of the database. This struct is never
/// serialized to the API directly: the transport layer maps it to a DTO that
/// excludes every encrypted field and the kubeconfig itself.
#[derive(Debug, Clone, Model)]
#[fabrique(table = "kubernetes.cluster")]
pub struct KubernetesCluster {
    #[fabrique(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub encrypted_kubeconfig: Vec<u8>,
    pub kubeconfig_nonce: Vec<u8>,
    pub dek_encrypted: Vec<u8>,
    pub dek_nonce: Vec<u8>,
    pub key_version: i32,
    pub encryption_algorithm: String,
    pub api_server_url: String,
    pub ca_fingerprint: Option<String>,
    pub kubernetes_version: Option<String>,
    pub platform: Option<String>,
    pub health_status: KubernetesClusterHealthStatus,
    pub last_health_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Non-sensitive information returned by a successful reachability check.
#[derive(Debug, Clone)]
pub struct ClusterHealthInfo {
    /// API server URL parsed from the kubeconfig current context.
    pub api_server_url: String,
    /// Kubernetes version reported by `GET /version` (e.g. `v1.32.2+k3s1`).
    /// Normalized to strict semver by the service layer before storage.
    pub kubernetes_version: String,
    /// OS/arch pair reported by the API server (e.g. `linux/amd64`).
    pub platform: String,
}

/// Reasons a cluster reachability check can fail.
#[derive(Debug, Error)]
pub enum ClusterHealthError {
    #[error("invalid kubeconfig: {0}")]
    InvalidKubeconfig(String),
    #[error("failed to build kubernetes client: {0}")]
    ClientBuild(String),
    #[error("cluster unreachable (timeout)")]
    Timeout,
    #[error("cluster credentials rejected: {0}")]
    Unauthorized(String),
    #[error("cluster unreachable: {0}")]
    Unreachable(String),
}

/// Reachability checker abstraction.
///
/// Injected into [`KubernetesClusters`] so the production path talks to a real
/// API server via kube-rs while tests provide a deterministic stub (there is no
/// real cluster in CI).
#[async_trait]
pub trait ClusterHealthChecker: Send + Sync {
    /// Verifies that the cluster described by `kubeconfig_yaml` is reachable,
    /// returning non-sensitive metadata (e.g. the API server URL) on success.
    async fn check(&self, kubeconfig_yaml: &str) -> Result<ClusterHealthInfo, ClusterHealthError>;
}

/// Production reachability checker: builds a kube-rs client from the supplied
/// kubeconfig and performs a short, RBAC-free `GET /version` request.
#[derive(Debug, Clone, Default)]
pub struct KubeHealthChecker;

#[async_trait]
impl ClusterHealthChecker for KubeHealthChecker {
    async fn check(&self, kubeconfig_yaml: &str) -> Result<ClusterHealthInfo, ClusterHealthError> {
        let mut config = parse_kubeconfig(kubeconfig_yaml)
            .await
            .map_err(ClusterHealthError::InvalidKubeconfig)?;
        config.connect_timeout = Some(CONNECT_TIMEOUT);
        config.read_timeout = Some(READ_TIMEOUT);

        let api_server_url = config.cluster_url.to_string();

        let client =
            Client::try_from(config).map_err(|e| ClusterHealthError::ClientBuild(e.to_string()))?;

        match timeout(HEALTH_CHECK_TIMEOUT, client.apiserver_version()).await {
            Err(_) => Err(ClusterHealthError::Timeout),
            Ok(Err(error)) => {
                let message = error.to_string();
                if message.contains("401") || message.contains("403") {
                    Err(ClusterHealthError::Unauthorized(message))
                } else {
                    Err(ClusterHealthError::Unreachable(message))
                }
            }
            Ok(Ok(info)) => Ok(ClusterHealthInfo {
                api_server_url,
                kubernetes_version: info.git_version,
                platform: info.platform,
            }),
        }
    }
}

/// Errors raised by the cluster service layer.
#[derive(Debug, Error)]
pub enum KubernetesClusterError {
    #[error("forbidden")]
    Forbidden,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("fabrique error: {0}")]
    Fabrique(#[from] fabrique::Error),
    #[error("cluster not found: {0}")]
    NotFound(Uuid),
    #[error("cluster name already exists: {0}")]
    NameAlreadyExists(String),
    #[error("invalid cluster name: {0}")]
    InvalidName(String),
    #[error("encryption error: {0}")]
    Encryption(#[from] frn_crypto::EncryptionError),
    #[error("health check failed: {0}")]
    HealthCheck(#[from] ClusterHealthError),
    #[error("stored kubeconfig is not valid UTF-8")]
    InvalidUtf8,
    #[error("cluster still hosts managed service instances and cannot be deleted: {0}")]
    ClusterHasInstances(Uuid),
    #[error("invalid kubeconfig: {0}")]
    InvalidKubeconfig(String),
    #[error("failed to build kubernetes client: {0}")]
    KubeClientBuild(String),
}

/// Fields accepted when registering a new cluster.
#[derive(Debug, Clone)]
pub struct CreateClusterInput {
    pub name: String,
    pub description: Option<String>,
    pub kubeconfig: String,
}

/// Fields accepted when updating a cluster. A `None` `kubeconfig` keeps the
/// existing credentials untouched and skips the reachability check.
#[derive(Debug, Clone)]
pub struct UpdateClusterInput {
    pub name: String,
    pub description: Option<String>,
    pub kubeconfig: Option<String>,
}

/// Service layer for the platform-admin Kubernetes cluster registry.
///
/// All operations are restricted to platform administrators
/// (`Principal::is_platform_admin`). The encryption key and the reachability
/// checker are injected at construction.
#[derive(Clone)]
pub struct KubernetesClusters {
    db: Pool<Postgres>,
    kek: Arc<Kek>,
    health_checker: Arc<dyn ClusterHealthChecker>,
}

impl KubernetesClusters {
    /// Builds the service with the production kube-rs reachability checker.
    pub fn new(db: Pool<Postgres>, kek: Arc<Kek>) -> Self {
        Self {
            db,
            kek,
            health_checker: Arc::new(KubeHealthChecker),
        }
    }

    /// Builds the service with a custom reachability checker (used by tests).
    pub fn with_health_checker(
        db: Pool<Postgres>,
        kek: Arc<Kek>,
        health_checker: Arc<dyn ClusterHealthChecker>,
    ) -> Self {
        Self {
            db,
            kek,
            health_checker,
        }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.db
    }

    /// Lists every registered cluster, most recent first.
    pub async fn list_clusters<P: Principal + Sync>(
        &self,
        principal: &P,
    ) -> Result<Vec<KubernetesCluster>, KubernetesClusterError> {
        require_admin(principal)?;
        KubernetesCluster::query()
            .select()
            .order_by(KubernetesCluster::CREATED_AT, Direction::Desc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Returns a single cluster by id.
    pub async fn get_cluster<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
    ) -> Result<KubernetesCluster, KubernetesClusterError> {
        require_admin(principal)?;
        self.find_cluster(cluster_id).await
    }

    /// Registers a new cluster after a successful reachability check.
    pub async fn create_cluster<P: Principal + Sync>(
        &self,
        principal: &P,
        input: CreateClusterInput,
    ) -> Result<KubernetesCluster, KubernetesClusterError> {
        let mut conn = self.db.acquire().await?;
        self.create_cluster_on(principal, input, &mut conn).await
    }

    /// Like [`Self::create_cluster`] but writes through the provided
    /// connection, so the caller can wrap the insert in a broader transaction
    /// (e.g. cluster creation + label attachment).
    pub async fn create_cluster_on<P: Principal + Sync>(
        &self,
        principal: &P,
        input: CreateClusterInput,
        conn: &mut PgConnection,
    ) -> Result<KubernetesCluster, KubernetesClusterError> {
        require_admin(principal)?;
        validate_name(&input.name)?;

        if self.find_by_name(&input.name).await?.is_some() {
            return Err(KubernetesClusterError::NameAlreadyExists(input.name));
        }

        let health = self.health_checker.check(&input.kubeconfig).await?;

        let cluster_id = Uuid::new_v4();
        let envelope = self.encrypt_kubeconfig(cluster_id, &input.kubeconfig)?;

        KubernetesCluster::query()
            .insert()
            .set(KubernetesCluster::ID, cluster_id)
            .set(KubernetesCluster::NAME, input.name)
            .set(KubernetesCluster::DESCRIPTION, input.description)
            .set(KubernetesCluster::ENCRYPTED_KUBECONFIG, envelope.ciphertext)
            .set(KubernetesCluster::KUBECONFIG_NONCE, envelope.nonce)
            .set(KubernetesCluster::DEK_ENCRYPTED, envelope.dek_ciphertext)
            .set(KubernetesCluster::DEK_NONCE, envelope.dek_nonce)
            .set(KubernetesCluster::KEY_VERSION, envelope.key_version)
            .set(
                KubernetesCluster::ENCRYPTION_ALGORITHM,
                frn_crypto::ALGORITHM.to_owned(),
            )
            .set(KubernetesCluster::API_SERVER_URL, health.api_server_url)
            .set(
                KubernetesCluster::KUBERNETES_VERSION,
                normalize_kubernetes_version(&health.kubernetes_version),
            )
            .set(KubernetesCluster::PLATFORM, Some(health.platform))
            .set(
                KubernetesCluster::HEALTH_STATUS,
                KubernetesClusterHealthStatus::Healthy,
            )
            .set(KubernetesCluster::LAST_HEALTH_CHECK_AT, Some(Utc::now()))
            .returning()
            .first(&mut *conn)
            .await?
            .ok_or(KubernetesClusterError::Database(sqlx::Error::RowNotFound))
    }

    /// Updates a cluster's metadata, re-checking and re-encrypting when a new
    /// kubeconfig is supplied.
    pub async fn update_cluster<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
        input: UpdateClusterInput,
    ) -> Result<KubernetesCluster, KubernetesClusterError> {
        require_admin(principal)?;
        validate_name(&input.name)?;

        let existing = self.find_cluster(cluster_id).await?;

        if let Some(other) = self.find_by_name(&input.name).await?
            && other.id != cluster_id
        {
            return Err(KubernetesClusterError::NameAlreadyExists(input.name));
        }

        let base_update = KubernetesCluster::query()
            .update()
            .set(KubernetesCluster::NAME, input.name)
            .set(KubernetesCluster::DESCRIPTION, input.description)
            .set(KubernetesCluster::UPDATED_AT, Utc::now());

        match input.kubeconfig {
            None => {
                base_update
                    .r#where(KubernetesCluster::ID, "=", cluster_id)
                    .execute(&self.db)
                    .await?;
            }
            Some(kubeconfig) => {
                let health = self.health_checker.check(&kubeconfig).await?;
                let envelope = self.encrypt_kubeconfig(existing.id, &kubeconfig)?;

                base_update
                    .set(KubernetesCluster::ENCRYPTED_KUBECONFIG, envelope.ciphertext)
                    .set(KubernetesCluster::KUBECONFIG_NONCE, envelope.nonce)
                    .set(KubernetesCluster::DEK_ENCRYPTED, envelope.dek_ciphertext)
                    .set(KubernetesCluster::DEK_NONCE, envelope.dek_nonce)
                    .set(KubernetesCluster::KEY_VERSION, envelope.key_version)
                    .set(
                        KubernetesCluster::ENCRYPTION_ALGORITHM,
                        frn_crypto::ALGORITHM.to_owned(),
                    )
                    .set(KubernetesCluster::API_SERVER_URL, health.api_server_url)
                    .set(
                        KubernetesCluster::KUBERNETES_VERSION,
                        normalize_kubernetes_version(&health.kubernetes_version),
                    )
                    .set(KubernetesCluster::PLATFORM, Some(health.platform))
                    .set(
                        KubernetesCluster::HEALTH_STATUS,
                        KubernetesClusterHealthStatus::Healthy,
                    )
                    .set(KubernetesCluster::LAST_HEALTH_CHECK_AT, Some(Utc::now()))
                    .r#where(KubernetesCluster::ID, "=", cluster_id)
                    .execute(&self.db)
                    .await?;
            }
        }

        self.find_cluster(cluster_id).await
    }

    /// Deletes a cluster within an explicit transaction: the active-instance
    /// guard, the cleanup of terminal "deleted" instances, and the cluster
    /// removal are atomic.
    pub async fn delete_cluster<P: Principal + Sync>(
        &self,
        principal: &P,
        cluster_id: Uuid,
    ) -> Result<(), KubernetesClusterError> {
        require_admin(principal)?;

        let mut tx = self.db.begin().await?;

        // Refuse to delete a cluster that still hosts active managed
        // service instances: removing it would orphan the deployed releases.
        // Callers must delete those instances first. Instances in the
        // terminal "deleted" FSM state are cleaned up below so the FK does
        // not block the cluster removal.
        let active_instance = ManagedServiceInstanceView::query()
            .select()
            .r#where(ManagedServiceInstanceView::CLUSTER_ID, "=", cluster_id)
            .r#where(
                ManagedServiceInstanceView::STATUS,
                "!=",
                ManagedServiceInstanceStatus::Deleted,
            )
            .first(&mut *tx)
            .await?;
        if active_instance.is_some() {
            return Err(KubernetesClusterError::ClusterHasInstances(cluster_id));
        }

        // Raw SQL: the DELETE requires a JOIN through the FSM tables to
        // resolve the abstract state name, which the query builder cannot
        // express.
        sqlx::query(
            "DELETE FROM managed.service_instance si
             USING lib_fsm.state_machine sm
             JOIN lib_fsm.abstract_state abs
                 ON abs.abstract_state__id = sm.abstract_state__id
             WHERE si.status = sm.state_machine__id
               AND si.cluster_id = $1
               AND abs.name = 'deleted'",
        )
        .bind(cluster_id)
        .execute(&mut *tx)
        .await?;

        let deleted = sqlx::query("DELETE FROM kubernetes.cluster WHERE id = $1")
            .bind(cluster_id)
            .execute(&mut *tx)
            .await?;

        if deleted.rows_affected() == 0 {
            return Err(KubernetesClusterError::NotFound(cluster_id));
        }

        tx.commit().await?;
        Ok(())
    }

    /// Decrypts and returns the kubeconfig for a cluster.
    ///
    /// This is the seam the deployment worker will use to build a per-cluster
    /// `kube::Client`. It is intentionally not exposed through the public CRUD
    /// API and performs no admin check: callers are trusted internal
    /// components, not external principals.
    pub async fn decrypt_kubeconfig(
        &self,
        cluster_id: Uuid,
    ) -> Result<String, KubernetesClusterError> {
        let cluster = self.find_cluster(cluster_id).await?;
        let aad = build_aad(cluster.id, cluster.key_version);
        let envelope = EnvelopeCiphertext {
            ciphertext: cluster.encrypted_kubeconfig,
            nonce: cluster.kubeconfig_nonce,
            dek_ciphertext: cluster.dek_encrypted,
            dek_nonce: cluster.dek_nonce,
            key_version: cluster.key_version,
        };
        let plaintext = frn_crypto::decrypt(&self.kek, &envelope, &aad)?;
        String::from_utf8(plaintext).map_err(|_| KubernetesClusterError::InvalidUtf8)
    }

    /// Picks a random healthy cluster carrying every required label, or
    /// `None` when no cluster matches.
    ///
    /// Used at managed-service deployment: the service's `deploy_target`
    /// declares the labels a hosting cluster must carry (e.g.
    /// `availability=ft`). Only healthy clusters are eligible so an instance
    /// is never bound to an unreachable one; when several clusters match, one
    /// is picked at random to spread the load. `required_labels` must not be
    /// empty: the caller is expected to reject services without a
    /// deploy_target before reaching this helper. This is an internal
    /// selection helper: it needs no KEK (no decryption) and performs no
    /// admin check, so it takes the pool directly rather than a constructed
    /// service.
    pub async fn pick_healthy_cluster_matching(
        db: &Pool<Postgres>,
        required_labels: &BTreeMap<String, String>,
    ) -> Result<Option<KubernetesCluster>, sqlx::Error> {
        let required = serde_json::to_value(required_labels)
            .expect("a map of strings always serializes to JSON");

        // Raw SQL: the query builder cannot express jsonb_each_text, the
        // double NOT EXISTS anti-join, or the ::citext parameter casts.
        // A cluster matches when no required pair is missing from its
        // attached labels; CITEXT columns make the comparison
        // case-insensitive. Clusters are ranked by hosted instance count
        // (least-loaded first) so new deployments spread across the fleet;
        // RANDOM() breaks ties.
        sqlx::query_as::<_, KubernetesCluster>(
            r#"SELECT c.*
               FROM kubernetes.cluster c
               WHERE c.health_status = 'healthy'
                 AND NOT EXISTS (
                     SELECT 1
                     FROM jsonb_each_text($1) AS required(key, value)
                     WHERE NOT EXISTS (
                         SELECT 1
                         FROM kubernetes.cluster_label cl
                         JOIN kubernetes.label l ON l.id = cl.label_id
                         WHERE cl.cluster_id = c.id
                           -- ::citext keeps the comparison case-insensitive:
                           -- without the cast Postgres resolves citext = text
                           -- to the case-sensitive text = text operator.
                           AND l.key = required.key::citext
                           AND l.value = required.value::citext
                     )
                 )
               ORDER BY (
                   SELECT COUNT(*)
                   FROM managed.service_instance si
                   WHERE si.cluster_id = c.id
               ), RANDOM()
               LIMIT 1"#,
        )
        .bind(required)
        .fetch_optional(db)
        .await
    }

    /// Builds a kube-rs client from a decrypted kubeconfig YAML.
    ///
    /// This is the seam the deployment worker uses to target a specific cluster:
    /// it decrypts the kubeconfig via [`Self::decrypt_kubeconfig`] and hands the
    /// YAML here to obtain a [`Client`], mirroring the wiring of the health
    /// checker.
    pub async fn client_from_kubeconfig(
        kubeconfig_yaml: &str,
    ) -> Result<Client, KubernetesClusterError> {
        let config = parse_kubeconfig(kubeconfig_yaml)
            .await
            .map_err(KubernetesClusterError::InvalidKubeconfig)?;
        Client::try_from(config)
            .map_err(|error| KubernetesClusterError::KubeClientBuild(error.to_string()))
    }

    fn encrypt_kubeconfig(
        &self,
        cluster_id: Uuid,
        kubeconfig: &str,
    ) -> Result<EnvelopeCiphertext, KubernetesClusterError> {
        let aad = build_aad(cluster_id, CURRENT_KEY_VERSION);
        frn_crypto::encrypt(&self.kek, kubeconfig.as_bytes(), &aad, CURRENT_KEY_VERSION)
            .map_err(Into::into)
    }

    async fn find_cluster(
        &self,
        cluster_id: Uuid,
    ) -> Result<KubernetesCluster, KubernetesClusterError> {
        KubernetesCluster::query()
            .select()
            .r#where(KubernetesCluster::ID, "=", cluster_id)
            .first(&self.db)
            .await?
            .ok_or(KubernetesClusterError::NotFound(cluster_id))
    }

    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<KubernetesCluster>, KubernetesClusterError> {
        KubernetesCluster::query()
            .select()
            .r#where(KubernetesCluster::NAME, "=", name.to_owned())
            .first(&self.db)
            .await
            .map_err(Into::into)
    }
}

/// Normalizes the version reported by `GET /version` into strict semver.
///
/// Kubernetes reports a `gitVersion` such as `v1.32.2+k3s1`: the leading `v`
/// is not valid semver and is stripped. Returns `None` when the remainder is
/// not parseable semver, so the database (whose CHECK enforces the format)
/// stores NULL instead of failing the whole create/update.
fn normalize_kubernetes_version(git_version: &str) -> Option<String> {
    let stripped = git_version.strip_prefix('v').unwrap_or(git_version);
    Version::parse(stripped)
        .ok()
        .map(|version| version.to_string())
}

/// Binds a ciphertext to its row by authenticating the cluster id and key
/// version as additional data.
fn build_aad(cluster_id: Uuid, key_version: i32) -> Vec<u8> {
    let mut aad = cluster_id.as_bytes().to_vec();
    aad.extend_from_slice(&key_version.to_be_bytes());
    aad
}

/// Rejects any principal that is not a platform administrator.
fn require_admin<P: Principal>(principal: &P) -> Result<(), KubernetesClusterError> {
    if principal.is_platform_admin() {
        Ok(())
    } else {
        Err(KubernetesClusterError::Forbidden)
    }
}

/// Parses a kubeconfig YAML string into a kube-rs [`Config`], factoring out the
/// two-step `Kubeconfig::from_yaml` + `Config::from_custom_kubeconfig` chain
/// shared by the health checker and the deployment client builder.
async fn parse_kubeconfig(yaml: &str) -> Result<Config, String> {
    let kubeconfig = Kubeconfig::from_yaml(yaml).map_err(|e| e.to_string())?;
    Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
        .await
        .map_err(|e| e.to_string())
}

/// Validates a cluster name against the DNS-label rules enforced by the
/// database CHECK constraint, so callers get a precise error instead of an
/// opaque constraint violation.
fn validate_name(name: &str) -> Result<(), KubernetesClusterError> {
    let invalid = || KubernetesClusterError::InvalidName(name.to_owned());
    if name.is_empty() || name.len() > MAX_NAME_LENGTH {
        return Err(invalid());
    }
    let bytes = name.as_bytes();
    let is_alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    if !is_alnum(bytes[0]) || !is_alnum(bytes[bytes.len() - 1]) {
        return Err(invalid());
    }
    if !name.bytes().all(|b| is_alnum(b) || b == b'-') {
        return Err(invalid());
    }
    Ok(())
}
