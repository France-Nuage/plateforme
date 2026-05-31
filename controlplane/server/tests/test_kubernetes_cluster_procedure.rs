//! Transport-layer tests for the Kubernetes cluster registry gRPC service.
//!
//! These cover authentication (the platform-admin gate), input validation, and
//! the synchronous reachability check rejecting an unreachable cluster. The
//! happy-path persistence/encryption logic is covered in
//! `test_kubernetes_cluster_service.rs` with a stubbed checker, since the gRPC
//! server uses the real kube-rs checker and CI has no live cluster.

mod common;

use common::{Api, WithUser, non_admin_token, seed_admin_token};
use frn_rpc::v1::kubernetes::{
    CreateClusterRequest, DeleteClusterRequest, GetClusterRequest, ListClustersRequest,
    UpdateClusterRequest,
};
use tonic::{Code, Request};
use uuid::Uuid;

const ADMIN_EMAIL: &str = "admin@francenuage.fr";

/// A syntactically valid kubeconfig pointing at an unroutable API server so the
/// reachability check fails fast with a connection error.
const UNREACHABLE_KUBECONFIG: &str = "apiVersion: v1\n\
kind: Config\n\
clusters:\n\
- name: test\n\
  cluster:\n\
    server: https://127.0.0.1:1\n\
contexts:\n\
- name: test\n\
  context:\n\
    cluster: test\n\
    user: test\n\
current-context: test\n\
users:\n\
- name: test\n\
  user:\n\
    token: fake-token\n";

#[sqlx::test(migrations = "../migrations")]
async fn list_requires_authentication(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .kubernetes
        .clusters
        .list_clusters(Request::new(ListClustersRequest {}))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn list_is_forbidden_for_non_admins(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = non_admin_token("regular@francenuage.fr");

    let response = api
        .kubernetes
        .clusters
        .list_clusters(Request::new(ListClustersRequest {}).with_user(&token))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::PermissionDenied);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn list_returns_empty_for_admin(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .list_clusters(Request::new(ListClustersRequest {}).with_user(&token))
        .await
        .expect("admin should be allowed to list");

    assert!(response.into_inner().clusters.is_empty());
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn create_rejects_empty_name(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .create_cluster(
            Request::new(CreateClusterRequest {
                name: String::new(),
                description: None,
                kubeconfig: UNREACHABLE_KUBECONFIG.to_owned(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn create_rejects_empty_kubeconfig(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .create_cluster(
            Request::new(CreateClusterRequest {
                name: "prod-eu".to_owned(),
                description: None,
                kubeconfig: "   ".to_owned(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn create_is_forbidden_for_non_admins(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = non_admin_token("regular@francenuage.fr");

    let response = api
        .kubernetes
        .clusters
        .create_cluster(
            Request::new(CreateClusterRequest {
                name: "prod-eu".to_owned(),
                description: None,
                kubeconfig: UNREACHABLE_KUBECONFIG.to_owned(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::PermissionDenied);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn create_fails_precondition_when_cluster_unreachable(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .create_cluster(
            Request::new(CreateClusterRequest {
                name: "prod-eu".to_owned(),
                description: None,
                kubeconfig: UNREACHABLE_KUBECONFIG.to_owned(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::FailedPrecondition);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn get_reports_not_found_for_unknown_cluster(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .get_cluster(
            Request::new(GetClusterRequest {
                cluster_id: Uuid::new_v4().to_string(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::NotFound);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn get_rejects_malformed_id(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .get_cluster(
            Request::new(GetClusterRequest {
                cluster_id: "not-a-uuid".to_owned(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_reports_not_found_for_unknown_cluster(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .delete_cluster(
            Request::new(DeleteClusterRequest {
                cluster_id: Uuid::new_v4().to_string(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::NotFound);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_requires_authentication(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .kubernetes
        .clusters
        .delete_cluster(Request::new(DeleteClusterRequest {
            cluster_id: Uuid::new_v4().to_string(),
        }))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn delete_is_forbidden_for_non_admins(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = non_admin_token("regular@francenuage.fr");

    let response = api
        .kubernetes
        .clusters
        .delete_cluster(
            Request::new(DeleteClusterRequest {
                cluster_id: Uuid::new_v4().to_string(),
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::PermissionDenied);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn update_requires_authentication(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .kubernetes
        .clusters
        .update_cluster(Request::new(UpdateClusterRequest {
            cluster_id: Uuid::new_v4().to_string(),
            name: "prod-eu".to_owned(),
            description: None,
            kubeconfig: None,
        }))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn update_is_forbidden_for_non_admins(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = non_admin_token("regular@francenuage.fr");

    let response = api
        .kubernetes
        .clusters
        .update_cluster(
            Request::new(UpdateClusterRequest {
                cluster_id: Uuid::new_v4().to_string(),
                name: "prod-eu".to_owned(),
                description: None,
                kubeconfig: None,
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::PermissionDenied);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn update_rejects_empty_name(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let response = api
        .kubernetes
        .clusters
        .update_cluster(
            Request::new(UpdateClusterRequest {
                cluster_id: Uuid::new_v4().to_string(),
                name: String::new(),
                description: None,
                kubeconfig: None,
            })
            .with_user(&token),
        )
        .await;

    assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);
    Ok(())
}
