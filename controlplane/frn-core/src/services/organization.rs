use sqlx::{Pool, Postgres};

use crate::{
    Error,
    models::{Organization, User},
};

pub async fn list(connection: &Pool<Postgres>, user: User) -> Result<Vec<Organization>, Error> {
    user.organizations(connection).await.map_err(Into::into)
}

pub async fn create(connection: &Pool<Postgres>, name: String) -> Result<Organization, Error> {
    Organization::factory()
        .name(name)
        .create(connection)
        .await
        .map_err(Into::into)
}
