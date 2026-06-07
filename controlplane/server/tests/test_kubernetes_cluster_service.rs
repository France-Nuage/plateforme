//! Service-layer tests for the Kubernetes cluster registry.
//!
//! These exercise the business logic directly with a stubbed reachability
//! checker (there is no real cluster in CI): encryption round-trips, the
//! platform-admin gate, name validation, uniqueness, and the full CRUD.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use fabrique::Factory;
use frn_core::identity::User;
use frn_core::kubernetes::{
    ClusterHealthChecker, ClusterHealthError, ClusterHealthInfo, CreateClusterInput,
    KubernetesClusterError, KubernetesClusterHealthStatus, KubernetesClusters, UpdateClusterInput,
};
use frn_core::resourcemanager::{Organization, Project};
use frn_crypto::Kek;
use uuid::Uuid;

const SAMPLE_KUBECONFIG: &str = "apiVersion: v1\nkind: Config\nclusters: []\n";
const API_SERVER_URL: &str = "https://cluster.example:6443/";
const KUBERNETES_VERSION: &str = "v1.32.2";
const PLATFORM: &str = "linux/amd64";

/// Deterministic reachability checker: returns success (with a fixed API
/// server URL) or a failure, without touching the network.
#[derive(Clone)]
struct StubChecker {
    fail: bool,
}

#[async_trait]
impl ClusterHealthChecker for StubChecker {
    async fn check(&self, _kubeconfig_yaml: &str) -> Result<ClusterHealthInfo, ClusterHealthError> {
        if self.fail {
            Err(ClusterHealthError::Unreachable("stub failure".to_owned()))
        } else {
            Ok(ClusterHealthInfo {
                api_server_url: API_SERVER_URL.to_owned(),
                kubernetes_version: KUBERNETES_VERSION.to_owned(),
                platform: PLATFORM.to_owned(),
            })
        }
    }
}

fn service(pool: &sqlx::PgPool, fail: bool) -> KubernetesClusters {
    KubernetesClusters::with_health_checker(
        pool.clone(),
        Arc::new(Kek::from_bytes([42u8; 32])),
        Arc::new(StubChecker { fail }),
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
    assert_eq!(cluster.kubernetes_version.as_deref(), Some(KUBERNETES_VERSION));
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

#[sqlx::test(migrations = "../migrations")]
async fn pick_random_healthy_cluster_returns_the_only_cluster(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let cluster = service
        .create_cluster(&user(true), create_input("prod-eu"))
        .await
        .expect("create");

    let picked = KubernetesClusters::pick_random_healthy_cluster(&pool)
        .await
        .expect("pick should not error")
        .expect("a healthy cluster should be returned");

    assert_eq!(picked.id, cluster.id);
    assert_eq!(picked.health_status, KubernetesClusterHealthStatus::Healthy);
}

#[sqlx::test(migrations = "../migrations")]
async fn pick_random_healthy_cluster_returns_none_when_empty(pool: sqlx::PgPool) {
    let picked = KubernetesClusters::pick_random_healthy_cluster(&pool)
        .await
        .expect("pick should not error");

    assert!(
        picked.is_none(),
        "no cluster should be returned when none exist"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_is_rejected_when_projects_are_assigned(pool: sqlx::PgPool) {
    let service = service(&pool, false);
    let admin = user(true);

    let cluster = service
        .create_cluster(&admin, create_input("prod-eu"))
        .await
        .expect("create");

    let organization = Organization::factory()
        .parent_id(None)
        .create(&pool)
        .await
        .expect("create organization");
    Project::factory()
        .organization_id(organization.id)
        .cluster_id(Some(cluster.id))
        .create(&pool)
        .await
        .expect("create project assigned to the cluster");

    let error = service
        .delete_cluster(&admin, cluster.id)
        .await
        .expect_err("deleting a cluster with assigned projects must be rejected");
    assert!(matches!(
        error,
        KubernetesClusterError::ClusterHasProjects(_)
    ));

    // The cluster must still exist.
    service
        .get_cluster(&admin, cluster.id)
        .await
        .expect("cluster must not have been deleted");
}
