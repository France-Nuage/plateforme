use crate::Error;
use crate::authorization::{Authorize, Principal};
use crate::resourcemanager::Organization;
use database::{Factory, Persistable, Repository};
use frn_derive::Resource;
use sqlx::{Pool, Postgres, types::chrono};
use uuid::Uuid;

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

    pub async fn find_or_create_one_by_email(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<User, sqlx::Error> {
        let maybe_user = User::find_one_by_email(pool, email).await?;

        match maybe_user {
            Some(user) => Ok(user),
            None => {
                User::factory()
                    .id(Uuid::new_v4())
                    .email(email.to_owned())
                    .create(pool)
                    .await
            }
        }
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

#[derive(Clone)]
pub struct Users<Auth: Authorize> {
    _auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Users<Auth> {
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { _auth: auth, db }
    }

    pub async fn find_or_create<P: Principal>(
        &self,
        principal: &P,
        email: String,
    ) -> Result<User, Error> {
        let maybe_user =
            sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1 LIMIT 1", &email)
                .fetch_optional(&self.db)
                .await?;

        match maybe_user {
            Some(user) => Ok(user),
            None => self.create(principal, email).await,
        }
    }

    pub async fn create<P: Principal>(&self, _principal: &P, email: String) -> Result<User, Error> {
        User::factory()
            .email(email)
            .is_admin(false)
            .create(&self.db)
            .await
            .map_err(Into::into)
    }
}
