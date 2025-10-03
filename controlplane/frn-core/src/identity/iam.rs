use crate::{Error, identity::User};

pub struct IAM {}

impl IAM {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn user(&self, _access_token: Option<String>) -> Result<User, Error> {
        Ok(User::default())
    }
}
