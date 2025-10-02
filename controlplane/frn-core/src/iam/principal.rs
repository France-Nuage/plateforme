use sqlx::{Pool, Postgres};

use crate::resourcemanager::Organization;

pub trait Principal {
    fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> impl Future<Output = Result<Vec<Organization>, crate::Error>>;
}
