use auth::model::User;
use database::{Factory, Persistable, Repository};
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::Problem;

#[derive(Debug, Default, Factory, FromRow, Repository)]
pub struct Organization {
    /// The organization id
    #[repository(primary)]
    pub id: Uuid,
    /// The organization name
    pub name: String,
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Organization {
    pub async fn find_by_user(
        pool: &Pool<Postgres>,
        user: User,
    ) -> Result<Vec<Organization>, Problem> {
        // Return all organizations if the user is an admin
        if user.is_admin {
            return Organization::list(pool).await.map_err(Into::into);
        }

        // Otherwise return the user organization
        sqlx::query_as!(
            Organization,
            r#"
                SELECT id, name, created_at, updated_at 
                FROM organizations 
                WHERE organizations.id = $1
            "#,
            user.organization_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}
