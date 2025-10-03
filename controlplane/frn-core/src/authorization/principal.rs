use sqlx::{Pool, Postgres};

use crate::{authorization::Resource, resourcemanager::Organization};

pub trait Principal: Resource + Sized {
    fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> impl Future<Output = Result<Vec<Organization>, crate::Error>>;
}
