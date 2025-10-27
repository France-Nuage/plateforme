//! Identity and access management
//!
//! Provides the `IAM` service for resolving user identity from access tokens.
//! Currently returns a default user; will be extended to validate OIDC tokens.

use crate::{
    Error,
    identity::{Principal, ServiceAccount, User},
};
use auth::OpenID;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct IAM {
    pub db: Pool<Postgres>,
    pub identity: OpenID,
}

impl IAM {
    pub fn new(db: Pool<Postgres>, identity: OpenID) -> Self {
        Self { db, identity }
    }

    pub async fn principal<T>(&self, request: &tonic::Request<T>) -> Result<Principal, Error> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(|value| value.to_owned())
            .ok_or(Error::Unauthenticated)?;

        println!("token: {:?}", &token);
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

        self.user(token).await.map(Principal::User)
    }

    async fn user(&self, access_token: String) -> Result<User, Error> {
        let email = self
            .identity
            .validate_token(&access_token)
            .await?
            .claims
            .email
            .ok_or(auth::Error::MissingEmailClaim)?;

        User::find_or_create_one_by_email(&self.db, &email)
            .await
            .map_err(Into::into)
    }
}
