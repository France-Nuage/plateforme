//! Transport-layer tests for the Profile gRPC service (`GetCurrentUser`).
//!
//! `GetCurrentUser` is the authoritative source of the frontend's platform-admin
//! status: it must reflect `users.is_admin` from the database, not an OIDC token
//! claim. These cover the authentication gate and that the admin flag and email
//! are reported from the resolved principal.

mod common;

use common::{Api, WithUser, non_admin_token, seed_admin_token};
use frn_rpc::v1::iam::GetCurrentUserRequest;
use tonic::{Code, Request};

const ADMIN_EMAIL: &str = "admin@francenuage.fr";
const REGULAR_EMAIL: &str = "regular@francenuage.fr";

#[sqlx::test(migrations = "../migrations")]
async fn get_current_user_requires_authentication(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");

    let response = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}))
        .await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn get_current_user_reports_non_admin(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = non_admin_token(REGULAR_EMAIL);

    let user = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_user(&token))
        .await
        .expect("authenticated user should resolve")
        .into_inner();

    assert!(!user.is_admin, "a freshly created user must not be admin");
    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn get_current_user_reports_admin(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::start(&pool).await.expect("could not start api");
    let token = seed_admin_token(&pool, ADMIN_EMAIL).await;

    let user = api
        .profile
        .get_current_user(Request::new(GetCurrentUserRequest {}).with_user(&token))
        .await
        .expect("authenticated admin should resolve")
        .into_inner();

    assert!(user.is_admin, "seeded admin must report is_admin=true");
    Ok(())
}
