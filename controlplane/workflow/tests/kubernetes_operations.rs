//! End-to-end tests for the Kubernetes operations against a real API server.
//!
//! These operations only talk to the API server (namespaces, secrets), so they
//! need neither a node nor a privileged container: a bare `kube-apiserver` +
//! `etcd` (the envtest assets) is enough, which runs on unprivileged CI runners.
//!
//! They are opt-in: `#[ignore]`d by default and gated on `E2E_KUBECONFIG`
//! pointing at a ready cluster. The `scripts/run-with-envtest.sh` helper spins
//! one up and sets `E2E_KUBECONFIG` for the wrapped command:
//!
//! ```text
//! scripts/run-with-envtest.sh cargo test -p workflow --test kubernetes_operations -- --ignored
//! ```
//!
//! Any real kubeconfig works too:
//!
//! ```text
//! E2E_KUBECONFIG=/path/to/kubeconfig cargo test -p workflow --test kubernetes_operations -- --ignored
//! ```
//!
//! Set `E2E_SUFFIX` (e.g. to `$CI_JOB_ID`) to make the created resource names
//! unique, so runs against a shared cluster do not collide.

use std::collections::BTreeMap;
use std::sync::Arc;

use k8s_openapi::api::core::v1::{Namespace, Secret};
use kube::Api;
use spicedb::SpiceDB;
use workflow::WorkerContext;
use workflow::execution::WorkflowExecutionId;
use workflow::operations::Operation;
use workflow::operations::assert_namespace_absent::AssertNamespaceAbsentOp;
use workflow::operations::create_k8s_secret::CreateK8sSecretOp;
use workflow::operations::create_namespace::CreateNamespaceOp;

/// Appends the optional `E2E_SUFFIX` so parallel runs or a shared cluster do
/// not race on a fixed name. Names stay DNS-1123 compliant (`CI_JOB_ID` is
/// numeric).
fn res_name(base: &str) -> String {
    match std::env::var("E2E_SUFFIX") {
        Ok(suffix) if !suffix.is_empty() => format!("{base}-{suffix}"),
        _ => base.to_owned(),
    }
}

async fn kube_client() -> kube::Client {
    let path = std::env::var("E2E_KUBECONFIG").expect("E2E_KUBECONFIG must point at a kubeconfig");
    let kubeconfig = kube::config::Kubeconfig::read_from(path).expect("could not read kubeconfig");
    let config = kube::Config::from_custom_kubeconfig(
        kubeconfig,
        &kube::config::KubeConfigOptions::default(),
    )
    .await
    .expect("could not build kube config");
    kube::Client::try_from(config).expect("could not build kube client")
}

async fn context_with(client: kube::Client) -> WorkerContext {
    let pool = sqlx::PgPool::connect_lazy("postgres://localhost:1/unused")
        .expect("could not build lazy pool");
    WorkerContext {
        pool,
        spicedb: SpiceDB::mock().await,
        kube: client,
        platform_config: workflow::PlatformConfig {
            default_storage_class: None,
            cnpg_backup_enabled: false,
        },
        kek: Arc::new(frn_crypto::Kek::from_bytes([42u8; 32])),
        kubeconfig_path: None,
    }
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn namespace_execute_creates_the_namespace() {
    let client = kube_client().await;
    let ctx = context_with(client.clone()).await;
    let namespace = res_name("frn-e2e-ns-create");

    CreateNamespaceOp {
        namespace: namespace.clone(),
        labels: BTreeMap::new(),
    }
    .execute(ctx, WorkflowExecutionId::new())
    .await
    .expect("namespace creation must succeed");

    let exists = Api::<Namespace>::all(client)
        .get_opt(&namespace)
        .await
        .expect("namespace lookup must succeed")
        .is_some();
    assert!(exists);
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn namespace_execute_is_idempotent() {
    let client = kube_client().await;
    let ctx = context_with(client).await;
    let namespace = res_name("frn-e2e-ns-idem");

    CreateNamespaceOp {
        namespace: namespace.clone(),
        labels: BTreeMap::new(),
    }
    .execute(ctx.clone(), WorkflowExecutionId::new())
    .await
    .expect("first namespace creation must succeed");

    let second = CreateNamespaceOp {
        namespace,
        labels: BTreeMap::new(),
    }
    .execute(ctx, WorkflowExecutionId::new())
    .await;

    assert!(second.is_ok());
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn secret_execute_creates_the_secret() {
    let client = kube_client().await;
    let ctx = context_with(client.clone()).await;
    let secret_name = res_name("frn-e2e-secret-create");
    let mut data = BTreeMap::new();
    data.insert("password".to_owned(), "s3cret".to_owned());

    CreateK8sSecretOp {
        namespace: "default".to_owned(),
        secret_name: secret_name.clone(),
        data,
    }
    .execute(ctx, WorkflowExecutionId::new())
    .await
    .expect("secret creation must succeed");

    let exists = Api::<Secret>::namespaced(client, "default")
        .get_opt(&secret_name)
        .await
        .expect("secret lookup must succeed")
        .is_some();
    assert!(exists);
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn secret_rollback_deletes_the_secret() {
    let client = kube_client().await;
    let ctx = context_with(client.clone()).await;
    let secret_name = res_name("frn-e2e-secret-rollback");
    let mut data = BTreeMap::new();
    data.insert("password".to_owned(), "s3cret".to_owned());

    let op = CreateK8sSecretOp {
        namespace: "default".to_owned(),
        secret_name: secret_name.clone(),
        data,
    };
    let executed = op
        .execute(ctx.clone(), WorkflowExecutionId::new())
        .await
        .expect("secret creation must succeed");
    executed
        .rollback(ctx, WorkflowExecutionId::new())
        .await
        .expect("secret rollback must succeed");

    let gone = Api::<Secret>::namespaced(client, "default")
        .get_opt(&secret_name)
        .await
        .expect("secret lookup must succeed")
        .is_none();
    assert!(gone);
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn assert_namespace_absent_succeeds_when_namespace_does_not_exist() {
    let client = kube_client().await;
    let ctx = context_with(client).await;
    let namespace = res_name("frn-e2e-ns-absent-ok");

    AssertNamespaceAbsentOp {
        namespace: namespace.clone(),
    }
    .execute(ctx, WorkflowExecutionId::new())
    .await
    .expect("assert absent must succeed for non-existing namespace");
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn assert_namespace_absent_fails_when_namespace_exists() {
    use workflow::operations::OperationError;
    use workflow::operations::assert_namespace_absent::AssertNamespaceAbsentError;

    let client = kube_client().await;
    let ctx = context_with(client).await;
    let namespace = res_name("frn-e2e-ns-absent-fail");

    CreateNamespaceOp {
        namespace: namespace.clone(),
        labels: BTreeMap::new(),
    }
    .execute(ctx.clone(), WorkflowExecutionId::new())
    .await
    .expect("namespace creation must succeed");

    let err = AssertNamespaceAbsentOp {
        namespace: namespace.clone(),
    }
    .execute(ctx, WorkflowExecutionId::new())
    .await
    .unwrap_err();

    assert!(matches!(err, AssertNamespaceAbsentError::AlreadyExists(_)));
    assert!(err.is_violated_invariant());
}

#[tokio::test]
#[ignore = "requires a Kubernetes cluster via E2E_KUBECONFIG"]
async fn assert_namespace_absent_rollback_is_a_noop() {
    let client = kube_client().await;
    let ctx = context_with(client).await;

    let result = AssertNamespaceAbsentOp {
        namespace: "frn-e2e-ns-absent-rollback".to_owned(),
    }
    .rollback(ctx, WorkflowExecutionId::new())
    .await;

    assert!(result.is_ok());
}
