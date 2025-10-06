//! Identity and access management
//!
//! Provides the `IAM` service for resolving user identity from access tokens.
//! Currently returns a default user; will be extended to validate OIDC tokens.

use crate::{Error, identity::User};

#[derive(Clone, Default)]
pub struct IAM {}

impl IAM {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn user(&self, _access_token: Option<String>) -> Result<User, Error> {
        Ok(User::default())
    }
}
