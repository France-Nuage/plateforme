use database::{Factory, Persistable, Repository};
use frn_derive::Resource;
use sqlx::{Pool, Postgres, types::chrono};
use uuid::Uuid;

use crate::{authorization::Principal, resourcemanager::Organization};

#[derive(Debug, Default, Factory, Repository, Resource)]
pub struct User {
    /// Unique identifier for the user
    #[repository(primary)]
    pub id: Uuid,

    /// The user email
    pub email: String,

    /// Administrative privileges flag.
    ///
    /// Indicates whether this user has administrative permissions within their
    /// organization. This field is part of the transitional authorization model
    /// and will be replaced by fine-grained SpiceDB permissions in the future.
    ///
    /// **Note**: This flag provides organization-scoped admin rights during the
    /// interim period before SpiceDB migration. Admin users may have elevated
    /// access to organization management functions.
    pub is_admin: bool,

    /// Creation time of the instance
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Time of the instance last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn find_one_by_email(
        pool: &sqlx::Pool<Postgres>,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, is_admin, created_at, updated_at 
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await
    }
}

impl Principal for User {
    /// Returns all organizations this user has access to
    async fn organizations(
        &self,
        connection: &Pool<Postgres>,
    ) -> Result<Vec<Organization>, crate::Error> {
        Organization::list(connection).await.map_err(Into::into)
    }
}
