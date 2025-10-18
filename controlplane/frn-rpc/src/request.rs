use frn_core::{
    authorization::Principal,
    identity::{IAM, ServiceAccount},
};

use crate::error::Error;

pub trait ExtractToken {
    fn access_token(&self) -> Option<String>;
    fn principal(&self, iam: IAM) -> impl Future<Output = Result<Box<dyn Principal>, Error>>;
}

impl<T> ExtractToken for tonic::Request<T> {
    fn access_token(&self) -> Option<String> {
        self.metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(|value| value.to_owned())
    }

    async fn principal(&self, iam: IAM) -> Result<Box<dyn Principal>, Error> {
        let token = self
            .access_token()
            .ok_or(Error::MissingAuthorizationHeader)?;

        if let Some(service_account) = sqlx::query_as!(
            ServiceAccount,
            "SELECT * from service_accounts WHERE key = $1",
            token
        )
        .fetch_optional(&iam.db)
        .await?
        {
            return Ok(Box::new(service_account));
        }

        iam.user(Some(token))
            .await
            .map(|u| Box::new(u) as Box<dyn Principal>)
            .map_err(Into::into)
    }
}
