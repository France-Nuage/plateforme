//! Authorization principals
//!
//! Defines the `Principal` trait for actors that can be subjects of
//! authorization checks. Principals are resources that can also query their
//! associated organizations from the database.

use crate::{authorization::Resource, resourcemanager::Organization};
use sqlx::{Pool, Postgres};

pub trait Principal: Resource + Send + Sync {
    /// Retrieve all organizations associated with this principal
    fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> impl Future<Output = Result<Vec<Organization>, crate::Error>>;
}
