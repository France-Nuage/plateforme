//! Service-layer tests for the Kubernetes cluster registry.
//!
//! These exercise the business logic directly with a stubbed reachability
//! checker (there is no real cluster in CI): encryption round-trips, the
//! platform-admin gate, name validation, uniqueness, and the full CRUD.

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use common::{attach_cluster_label, seed_managed_service, seed_managed_service_version};
use fabrique::Query;
use frn_core::identity::User;
use frn_core::kubernetes::{
    ClusterHealthChecker, ClusterHealthError, ClusterHealthInfo, CreateClusterInput,
    KubernetesCluster, KubernetesClusterError, KubernetesClusterHealthStatus, KubernetesClusters,
    UpdateClusterInput,
};
use frn_core::managed::ManagedServiceInstance;
use frn_crypto::Kek;
use uuid::Uuid;

const SAMPLE_KUBECONFIG: &str = "apiVersion: v1\nkind: Config\nclusters: []\n";
const API_SERVER_URL: &str = "https://cluster.example:6443/";
const KUBERNETES_VERSION: &str = "v1.32.2";
/// What the service stores after stripping the non-semver `v` prefix.
const NORMALIZED_KUBERNETES_VERSION: &str = "1.32.2";
const PLATFORM: &str = "linux/amd64";

/// Deterministic reachability checker: returns success (with configurable API
/// server URL and reported version) or a failure, without touching the
/// network.
#[derive(Clone)]
struct StubChecker {
    fail: bool,
    kubernetes_version: String,
    api_server_url: String,
}

#[async_trait]
impl ClusterHealthChecker for StubChecker {
    async fn check(&self, _kubeconfig_yaml: &str) -> Result<ClusterHealthInfo, ClusterHealthError> {
        if self.fail {
            Err(ClusterHealthError::Unreachable("stub failure".to_owned()))
        } else {
            Ok(ClusterHealthInfo {
                api_server_url: self.api_server_url.clone(),
                kubernetes_version: self.kubernetes_version.clone(),
                platform: PLATFORM.to_owned(),
            })
        }
    }
}

fn service(pool: &sqlx::PgPool, fail: bool) -> KubernetesClusters {
    service_with_version(pool, fail, KUBERNETES_VERSION)
}

fn service_with_version(pool: &sqlx::PgPool, fail: bool, version: &str) -> KubernetesClusters {
    KubernetesClusters::with_health_checker(
        pool.clone(),
        Arc::new(Kek::from_bytes([42u8; 32])),
        Arc::new(StubChecker {
            fail,
            kubernetes_version: version.to_owned(),
            api_server_url: API_SERVER_URL.to_owned(),
        }),
    )
}

/// Service whose stub reports a distinct API server URL, so tests can register
/// several clusters without tripping the api_server_url unique constraint.
fn service_with_url(pool: &sqlx::PgPool, api_server_url: &str) -> KubernetesClusters {
    KubernetesClusters::with_health_checker(
        pool.clone(),
        Arc::new(Kek::from_bytes([42u8; 32])),
        Arc::new(StubChecker {
            fail: false,
            kubernetes_version: KUBERNETES_VERSION.to_owned(),
            api_server_url: api_server_url.to_owned(),
        }),
    )
}

fn user(is_admin: bool) -> User {
    User {
        id: Uuid::new_v4(),
        email: format!("{}@francenuage.fr", if is_admin { "admin" } else { "user" }),
        is_admin,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn create_input(name: &str) -> CreateClusterInput {
    CreateClusterInput {
        name: name.to_owned(),
        description: Some("test cluster".to_owned()),
        kubeconfig: SAMPLE_KUBECONFIG.to_owned(),
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn create_persists_encrypts_and_round_trips(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("cluster should be created");

    assert_eq!(cluster.name, "prod-eu");
    assert_eq!(cluster.api_server_url, API_SERVER_URL);
    assert_eq!(
        cluster.kubernetes_version.as_deref(),
        Some(NORMALIZED_KUBERNETES_VERSION)
    );
    assert_eq!(cluster.platform.as_deref(), Some(PLATFORM));
    assert_eq!(
        cluster.health_status,
        KubernetesClusterHealthStatus::Healthy
    );
    assert!(cluster.last_health_check_at.is_some());

    // The kubeconfig is stored encrypted, never as plaintext.
    assert!(!cluster.encrypted_kubeconfig.is_empty());
    assert_ne!(cluster.encrypted_kubeconfig, SAMPLE_KUBECONFIG.as_bytes());

    // Decryption returns the original kubeconfig.
    let decrypted = service
        .decrypt_kubeconfig(cluster.id)
        .await
        .expect("kubeconfig should decrypt");
    assert_eq!(decrypted, SAMPLE_KUBECONFIG);

    // Get returns the persisted cluster.
    let fetched = service
        .get_cluster(&admin, cluster.id)
        .await
        .expect("cluster should be retrievable");
    assert_eq!(fetched.id, cluster.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn create_is_rejected_for_non_admins(pool: sqlx::PgPool) {
    let service = service(&pool, false);

    let error = service
        .create_cluster(&user(false), create_input("prod-eu"))
        .await
        .expect_err("non-admin must be forbidden");

    assert!(matches!(error, KubernetesClusterError::Forbidden));
}

#[sqlx::test(migrations = "../migrations")]
async fn create_rejects_invalid_names(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    for invalid in ["Prod EU", "-leading", "trailing-", "UPPER", ""] {
        let error = service
            .create_cluster(&admin, create_input(invalid))
            .await
            .expect_err("invalid name must be rejected");
        assert!(
            matches!(error, KubernetesClusterError::InvalidName(_)),
            "name {invalid:?} should be InvalidName, got {error:?}"
        );
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn create_rejects_duplicate_names(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("first create should succeed");

    let error = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect_err("duplicate name must be rejected");

    assert!(matches!(
        error,
        KubernetesClusterError::NameAlreadyExists(_)
    ));
}

#[sqlx::test(migrations = "../migrations")]
async fn create_fails_when_cluster_is_unreachable(pool: sqlx::PgPool) {
    let service = service(&pool, true);
    let admin = user(true);

    let error = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect_err("unreachable cluster must fail the health check");

    assert!(matches!(error, KubernetesClusterError::HealthCheck(_)));

    // Nothing should have been persisted.
    let clusters = service.list_clusters(&admin).await.expect("list");
    assert!(clusters.is_empty());
}

#[sqlx::test(migrations = "../migrations")]
async fn create_normalizes_kubernetes_version_to_semver(pool: sqlx::PgPool) {
    let service = service_with_version(&pool, false, "v1.31.4+k3s1");
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("cluster should be created");

    // The non-semver "v" prefix is stripped, build metadata is preserved.
    assert_eq!(cluster.kubernetes_version.as_deref(), Some("1.31.4+k3s1"));
}

#[sqlx::test(migrations = "../migrations")]
async fn create_clears_non_semver_kubernetes_version(pool: sqlx::PgPool) {
    let service = service_with_version(&pool, false, "not-a-version");
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("cluster should be created despite the exotic version");

    // An unparseable version is stored as NULL, never raw (the database CHECK
    // only accepts strict semver).
    assert_eq!(cluster.kubernetes_version, None);
}

#[sqlx::test(migrations = "../migrations")]
async fn update_with_new_kubeconfig_normalizes_kubernetes_version(pool: sqlx::PgPool) {
    let admin = user(true);
    let cluster = service(&pool, false)
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create");

    let updated = service_with_version(&pool, false, "v1.33.0-rc.1+vmware.1")
        .update_cluster(
            &admin,
            cluster.id,
            UpdateClusterInput {
                name: "prod-eu".to_owned(),
                description: None,
                kubeconfig: Some(SAMPLE_KUBECONFIG.to_owned()),
            },
        )
        .await
        .expect("update with new kubeconfig");

    assert_eq!(
        updated.kubernetes_version.as_deref(),
        Some("1.33.0-rc.1+vmware.1")
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn update_metadata_keeps_kubeconfig(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create");

    let updated = service
        .update_cluster(
            &admin,
            cluster.id,
            UpdateClusterInput {
                name: "prod-eu-renamed".to_owned(),
                description: None,
                kubeconfig: None,
            },
        )
        .await
        .expect("update");

    assert_eq!(updated.name, "prod-eu-renamed");
    assert_eq!(updated.description, None);

    let decrypted = service
        .decrypt_kubeconfig(cluster.id)
        .await
        .expect("decrypt");
    assert_eq!(decrypted, SAMPLE_KUBECONFIG);
}

#[sqlx::test(migrations = "../migrations")]
async fn update_with_new_kubeconfig_reencrypts(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create");

    let new_kubeconfig = "apiVersion: v1\nkind: Config\nclusters: [updated]\n";
    service
        .update_cluster(
            &admin,
            cluster.id,
            UpdateClusterInput {
                name: "prod-eu".to_owned(),
                description: Some("rotated".to_owned()),
                kubeconfig: Some(new_kubeconfig.to_owned()),
            },
        )
        .await
        .expect("update with new kubeconfig");

    let decrypted = service
        .decrypt_kubeconfig(cluster.id)
        .await
        .expect("decrypt");
    assert_eq!(decrypted, new_kubeconfig);
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_removes_cluster_and_reports_missing(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create");

    service
        .delete_cluster(&admin, cluster.id)
        .await
        .expect("delete");

    let missing = service
        .delete_cluster(&admin, cluster.id)
        .await
        .expect_err("deleting again must report not found");
    assert!(matches!(missing, KubernetesClusterError::NotFound(_)));

    let get_missing = service
        .get_cluster(&admin, cluster.id)
        .await
        .expect_err("get must report not found");
    assert!(matches!(get_missing, KubernetesClusterError::NotFound(_)));
}

#[sqlx::test(migrations = "../migrations")]
async fn list_is_rejected_for_non_admins(pool: sqlx::PgPool) {
    let service = service(&pool, false);

    let error = service
        .list_clusters(&user(false))
        .await
        .expect_err("non-admin must be forbidden");

    assert!(matches!(error, KubernetesClusterError::Forbidden));
}

/// Shorthand: builds the required-labels map of a deploy_target.
fn required(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_matching_returns_the_cluster_carrying_all_labels(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let matching = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create matching cluster");
    let other = service_with_url(&pool, "https://other.example:6443/")
        .create_cluster(
            &admin,
            CreateClusterInput {
                name: "prod-us".to_owned(),
                description: None,
                kubeconfig: SAMPLE_KUBECONFIG.to_owned(),
            },
        )
        .await
        .expect("create other cluster");

    attach_cluster_label(&pool, matching.id, "availability", "ft").await;
    attach_cluster_label(&pool, matching.id, "region", "fr").await;
    attach_cluster_label(&pool, other.id, "availability", "standard").await;

    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft"), ("region", "fr")]),
    )
    .await
    .expect("pick should not error")
    .expect("the labeled cluster should be returned");

    assert_eq!(picked.id, matching.id);
    assert_eq!(picked.health_status, KubernetesClusterHealthStatus::Healthy);
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_matching_requires_every_label(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let cluster = service
        .create_cluster(&user(true), create_input("prod-eu"))
        .await
        .expect("create");

    // The cluster carries only one of the two required labels.
    attach_cluster_label(&pool, cluster.id, "availability", "ft").await;

    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft"), ("region", "fr")]),
    )
    .await
    .expect("pick should not error");
    assert!(
        picked.is_none(),
        "a cluster missing one required label must not match"
    );

    // Attaching the missing label makes it eligible.
    attach_cluster_label(&pool, cluster.id, "region", "fr").await;
    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft"), ("region", "fr")]),
    )
    .await
    .expect("pick should not error")
    .expect("the cluster now carries every required label");
    assert_eq!(picked.id, cluster.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_matching_returns_none_when_no_cluster_exists(pool: sqlx::PgPool) {
    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft")]),
    )
    .await
    .expect("pick should not error");

    assert!(
        picked.is_none(),
        "no cluster should be returned when none exist"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_matching_ignores_unhealthy_clusters(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let cluster = service
        .create_cluster(&user(true), create_input("prod-eu"))
        .await
        .expect("create");
    attach_cluster_label(&pool, cluster.id, "availability", "ft").await;

    KubernetesCluster::query()
        .update()
        .set(
            KubernetesCluster::HEALTH_STATUS,
            KubernetesClusterHealthStatus::Unreachable,
        )
        .r#where(KubernetesCluster::ID, "=", cluster.id)
        .execute(&pool)
        .await
        .expect("mark cluster unreachable");

    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft")]),
    )
    .await
    .expect("pick should not error");

    assert!(
        picked.is_none(),
        "an unreachable cluster must never be picked even when its labels match"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_matching_picks_among_all_candidates(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let first = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create first cluster");
    let second = service_with_url(&pool, "https://other.example:6443/")
        .create_cluster(
            &admin,
            CreateClusterInput {
                name: "prod-us".to_owned(),
                description: None,
                kubeconfig: SAMPLE_KUBECONFIG.to_owned(),
            },
        )
        .await
        .expect("create second cluster");
    attach_cluster_label(&pool, first.id, "availability", "ft").await;
    attach_cluster_label(&pool, second.id, "availability", "ft").await;

    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft")]),
    )
    .await
    .expect("pick should not error")
    .expect("one of the matching clusters should be returned");

    assert!(
        picked.id == first.id || picked.id == second.id,
        "the picked cluster must be one of the matching candidates"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_matching_is_case_insensitive(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let cluster = service
        .create_cluster(&user(true), create_input("prod-eu"))
        .await
        .expect("create");

    // CITEXT columns: AVAILABILITY=FT and availability=ft are the same label.
    attach_cluster_label(&pool, cluster.id, "AVAILABILITY", "FT").await;

    let picked = KubernetesClusters::pick_healthy_cluster_matching(
        &pool,
        &required(&[("availability", "ft")]),
    )
    .await
    .expect("pick should not error")
    .expect("matching must ignore the casing of keys and values");

    assert_eq!(picked.id, cluster.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_is_rejected_when_instances_are_hosted(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create");

    // A managed service instance hosted on the cluster (inserted directly:
    // the full creation flow is covered by the instance procedure tests).
    let service_id = seed_managed_service(&pool, "vaultwarden", "Vaultwarden", "security").await;
    let version_id = seed_managed_service_version(
        &pool,
        service_id,
        "1.0.0",
        None,
        "oci://registry.test/charts/vaultwarden",
    )
    .await;
    ManagedServiceInstance::query()
        .insert()
        .set(ManagedServiceInstance::SERVICE_ID, service_id)
        .set(ManagedServiceInstance::VERSION_ID, version_id)
        .set(
            ManagedServiceInstance::PROJECT_SLUG,
            "test-project".to_owned(),
        )
        .set(
            ManagedServiceInstance::ORGANIZATION_SLUG,
            "test-org".to_owned(),
        )
        .set(ManagedServiceInstance::CLUSTER_ID, cluster.id)
        .set(
            ManagedServiceInstance::NAMESPACE,
            "managed-acme-vaultwarden-prod".to_owned(),
        )
        .set(
            ManagedServiceInstance::RELEASE_NAME,
            "vaultwarden".to_owned(),
        )
        .execute(&pool)
        .await
        .expect("insert instance hosted on the cluster");

    let error = service
        .delete_cluster(&admin, cluster.id)
        .await
        .expect_err("deleting a cluster hosting instances must be rejected");
    assert!(matches!(
        error,
        KubernetesClusterError::ClusterHasInstances(_)
    ));

    // The cluster must still exist.
    service
        .get_cluster(&admin, cluster.id)
        .await
        .expect("cluster must not have been deleted");
}
