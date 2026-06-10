//! Service-layer tests for the Kubernetes cluster label registry.
//!
//! These exercise the business logic directly: the platform-admin gate,
//! key/value validation, uniqueness, system-label protection, and the
//! attach/detach lifecycle.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use common::attach_cluster_label;
use fabrique::Factory;
use frn_core::identity::User;
use frn_core::kubernetes::{
    ClusterHealthChecker, ClusterHealthError, ClusterHealthInfo, CreateClusterInput,
    KubernetesCluster, KubernetesClusters, KubernetesLabel, KubernetesLabelError, KubernetesLabels,
};
use frn_crypto::Kek;
use uuid::Uuid;

fn labels(pool: &sqlx::PgPool) -> KubernetesLabels {
    KubernetesLabels::new(pool.clone())
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

/// Reachability checker stub used to seed clusters without a real API server.
#[derive(Clone)]
struct StubChecker;

#[async_trait]
impl ClusterHealthChecker for StubChecker {
    async fn check(&self, _kubeconfig_yaml: &str) -> Result<ClusterHealthInfo, ClusterHealthError> {
        Ok(ClusterHealthInfo {
            api_server_url: "https://cluster.example:6443/".to_owned(),
            kubernetes_version: "v1.32.2".to_owned(),
            platform: "linux/amd64".to_owned(),
        })
    }
}

async fn seed_cluster(pool: &sqlx::PgPool, name: &str) -> KubernetesCluster {
    let service = KubernetesClusters::with_health_checker(
        pool.clone(),
        Arc::new(Kek::from_bytes([42u8; 32])),
        Arc::new(StubChecker),
    );
    service
        .create_cluster(
            &user(true),
            CreateClusterInput {
                name: name.to_owned(),
                description: None,
                kubeconfig: "apiVersion: v1\nkind: Config\nclusters: []\n".to_owned(),
            },
        )
        .await
        .expect("could not seed cluster")
}

/// Inserts a control-plane-owned label directly (the API refuses to create
/// system labels, so tests must seed them through the database).
async fn seed_system_label(pool: &sqlx::PgPool, key: &str, value: &str) -> Uuid {
    KubernetesLabel::factory()
        .key(key.to_owned())
        .value(value.to_owned())
        .system(true)
        .create(pool)
        .await
        .expect("could not seed system label")
        .id
}

#[sqlx::test(migrations = "../migrations")]
async fn create_and_list_labels(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);

    let created = service
        .create_label(&admin, "availability".to_owned(), "ft".to_owned())
        .await
        .expect("label should be created");
    assert_eq!(created.key, "availability");
    assert_eq!(created.value, "ft");
    assert!(
        !created.system,
        "API-created labels are never system labels"
    );

    let all = service.list_labels(&admin).await.expect("list");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, created.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn create_rejects_duplicate_even_with_different_casing(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);

    service
        .create_label(&admin, "availability".to_owned(), "ft".to_owned())
        .await
        .expect("first label should be created");

    // CITEXT: AVAILABILITY=FT is the same label as availability=ft.
    let error = service
        .create_label(&admin, "AVAILABILITY".to_owned(), "FT".to_owned())
        .await
        .expect_err("duplicate label must be rejected regardless of casing");
    assert!(matches!(error, KubernetesLabelError::AlreadyExists { .. }));
}

#[sqlx::test(migrations = "../migrations")]
async fn create_rejects_invalid_keys_and_values(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);

    for invalid in ["", "with space", "with_underscore", "été", &"a".repeat(50)] {
        let error = service
            .create_label(&admin, invalid.to_owned(), "ft".to_owned())
            .await
            .expect_err("invalid key must be rejected");
        assert!(
            matches!(error, KubernetesLabelError::InvalidKey(_)),
            "key '{invalid}' must be rejected as InvalidKey"
        );

        let error = service
            .create_label(&admin, "availability".to_owned(), invalid.to_owned())
            .await
            .expect_err("invalid value must be rejected");
        assert!(
            matches!(error, KubernetesLabelError::InvalidValue(_)),
            "value '{invalid}' must be rejected as InvalidValue"
        );
    }

    // 49 chars is the maximum accepted length.
    service
        .create_label(&admin, "a".repeat(49), "b".repeat(49))
        .await
        .expect("49-char key and value must be accepted");
}

#[sqlx::test(migrations = "../migrations")]
async fn label_operations_are_rejected_for_non_admins(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let outsider = user(false);
    let label_id = Uuid::new_v4();
    let cluster_id = Uuid::new_v4();

    assert!(matches!(
        service.list_labels(&outsider).await.expect_err("forbidden"),
        KubernetesLabelError::Forbidden
    ));
    assert!(matches!(
        service
            .create_label(&outsider, "availability".to_owned(), "ft".to_owned())
            .await
            .expect_err("forbidden"),
        KubernetesLabelError::Forbidden
    ));
    assert!(matches!(
        service
            .delete_label(&outsider, label_id)
            .await
            .expect_err("forbidden"),
        KubernetesLabelError::Forbidden
    ));
    assert!(matches!(
        service
            .attach_label(&outsider, cluster_id, label_id)
            .await
            .expect_err("forbidden"),
        KubernetesLabelError::Forbidden
    ));
    assert!(matches!(
        service
            .detach_label(&outsider, cluster_id, label_id)
            .await
            .expect_err("forbidden"),
        KubernetesLabelError::Forbidden
    ));
}

#[sqlx::test(migrations = "../migrations")]
async fn attach_and_detach_lifecycle(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);
    let cluster = seed_cluster(&pool, "prod-eu").await;

    let label = service
        .create_label(&admin, "availability".to_owned(), "ft".to_owned())
        .await
        .expect("create label");

    service
        .attach_label(&admin, cluster.id, label.id)
        .await
        .expect("attach should succeed");
    // Idempotent: attaching twice is not an error.
    service
        .attach_label(&admin, cluster.id, label.id)
        .await
        .expect("re-attaching must be idempotent");

    let attached = service
        .list_cluster_labels(&admin, cluster.id)
        .await
        .expect("list cluster labels");
    assert_eq!(attached.len(), 1);
    assert_eq!(attached[0].id, label.id);

    service
        .detach_label(&admin, cluster.id, label.id)
        .await
        .expect("detach should succeed");
    // Idempotent: detaching a label that is not attached is not an error.
    service
        .detach_label(&admin, cluster.id, label.id)
        .await
        .expect("re-detaching must be idempotent");

    let attached = service
        .list_cluster_labels(&admin, cluster.id)
        .await
        .expect("list cluster labels");
    assert!(attached.is_empty());
}

#[sqlx::test(migrations = "../migrations")]
async fn attach_rejects_unknown_cluster_and_label(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);
    let cluster = seed_cluster(&pool, "prod-eu").await;

    let error = service
        .attach_label(&admin, cluster.id, Uuid::new_v4())
        .await
        .expect_err("attaching an unknown label must fail");
    assert!(matches!(error, KubernetesLabelError::NotFound(_)));

    let label = service
        .create_label(&admin, "availability".to_owned(), "ft".to_owned())
        .await
        .expect("create label");
    let error = service
        .attach_label(&admin, Uuid::new_v4(), label.id)
        .await
        .expect_err("attaching to an unknown cluster must fail");
    assert!(matches!(error, KubernetesLabelError::ClusterNotFound(_)));
}

#[sqlx::test(migrations = "../migrations")]
async fn system_labels_are_read_only_through_the_api(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);
    let cluster = seed_cluster(&pool, "prod-eu").await;
    let system_label_id = seed_system_label(&pool, "creator", "frn").await;

    let error = service
        .delete_label(&admin, system_label_id)
        .await
        .expect_err("deleting a system label must be rejected even for admins");
    assert!(matches!(
        error,
        KubernetesLabelError::SystemLabelReadOnly(_)
    ));

    let error = service
        .attach_label(&admin, cluster.id, system_label_id)
        .await
        .expect_err("attaching a system label must be rejected even for admins");
    assert!(matches!(
        error,
        KubernetesLabelError::SystemLabelReadOnly(_)
    ));

    let error = service
        .detach_label(&admin, cluster.id, system_label_id)
        .await
        .expect_err("detaching a system label must be rejected even for admins");
    assert!(matches!(
        error,
        KubernetesLabelError::SystemLabelReadOnly(_)
    ));

    // System labels remain visible in listings.
    let all = service.list_labels(&admin).await.expect("list");
    assert_eq!(all.len(), 1);
    assert!(all[0].system);
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_label_removes_its_attachments(pool: sqlx::PgPool) {
    let service = labels(&pool);
    let admin = user(true);
    let cluster = seed_cluster(&pool, "prod-eu").await;

    let label = service
        .create_label(&admin, "availability".to_owned(), "ft".to_owned())
        .await
        .expect("create label");
    attach_cluster_label(&pool, cluster.id, "availability", "ft").await;

    service
        .delete_label(&admin, label.id)
        .await
        .expect("delete should succeed");

    let attached = service
        .list_cluster_labels(&admin, cluster.id)
        .await
        .expect("list cluster labels");
    assert!(
        attached.is_empty(),
        "deleting a label must cascade to its attachments"
    );
    let error = service
        .delete_label(&admin, label.id)
        .await
        .expect_err("deleting twice must report NotFound");
    assert!(matches!(error, KubernetesLabelError::NotFound(_)));
}
