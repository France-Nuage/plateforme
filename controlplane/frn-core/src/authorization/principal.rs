//! Authorization principals
//!
//! Defines the `Principal` trait for actors that can be subjects of
//! authorization checks. Principals are resources that can also query their
//! associated organizations from the database.

use crate::{authorization::Resource, resourcemanager::Organization};
use sqlx::{Pool, Postgres};
use std::fmt::Debug;

pub trait Principal: Resource + Debug + Send + Sync {
    /// Retrieve all organizations associated with this principal
    fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> impl Future<Output = Result<Vec<Organization>, crate::Error>>;

    /// Whether this principal holds platform-administration privileges.
    ///
    /// Defaults to `false`; overridden by [`crate::identity::User`] to expose
    /// its `is_admin` flag. Gates access to platform-level resources that are
    /// not scoped to a specific organization.
    fn is_platform_admin(&self) -> bool {
        false
    }
}
