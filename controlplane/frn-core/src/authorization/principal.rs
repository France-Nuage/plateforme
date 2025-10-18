//! Authorization principals
//!
//! Defines the `Principal` trait for actors that can be subjects of
//! authorization checks. Principals are resources that can also query their
//! associated organizations from the database.

use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use crate::{authorization::Resource, resourcemanager::Organization};

#[async_trait]
pub trait Principal: Resource + Send + Sync {
    async fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> Result<Vec<Organization>, crate::Error>;
}
