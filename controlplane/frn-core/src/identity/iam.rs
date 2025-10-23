//! Identity and access management
//!
//! Provides the `IAM` service for resolving user identity from access tokens.
//! Currently returns a default user; will be extended to validate OIDC tokens.

use crate::{
    Error,
    identity::{Principal, ServiceAccount, User},
};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct IAM {
    pub db: Pool<Postgres>,
}

impl IAM {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn principal<T>(&self, request: &tonic::Request<T>) -> Result<Principal, Error> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(|value| value.to_owned())
            .ok_or(Error::Unauthenticated)?;

        if let Some(service_account) = sqlx::query_as!(
            ServiceAccount,
            "SELECT * from service_accounts WHERE key = $1",
            token
        )
        .fetch_optional(&self.db)
        .await?
        {
            return Ok(Principal::ServiceAccount(service_account));
        }

        self.user(Some(token)).await.map(Principal::User)
    }

    pub async fn user(&self, _access_token: Option<String>) -> Result<User, Error> {
        Ok(User::default())
    }
}
