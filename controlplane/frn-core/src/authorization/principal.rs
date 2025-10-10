//! Authorization principals
//!
//! Defines the `Principal` trait for actors that can be subjects of
//! authorization checks. Principals are resources that can also query their
//! associated organizations from the database.

use sqlx::{Pool, Postgres};

use crate::{authorization::Resource, resourcemanager::Organization};

pub trait Principal: Resource + Sized + Sync {
    fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> impl Future<Output = Result<Vec<Organization>, crate::Error>>;
}
