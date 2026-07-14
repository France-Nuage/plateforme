use crate::Error;
use crate::authorization::{Authorize, Principal};
use crate::resourcemanager::Organization;
use fabrique::{Factory, Model, Persist, Query};
use frn_derive::Resource;
use sqlx::{Pool, Postgres, types::chrono};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct User {
    /// Unique identifier for the user
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The user email
    pub email: String,

    /// Platform-administration flag.
    ///
    /// Indicates whether this user holds platform-wide administrative
    /// privileges. This field is part of the transitional authorization model
    /// and will be replaced by fine-grained SpiceDB permissions in the future.
    ///
    /// **Note**: despite its historical name, this flag grants platform-level
    /// access (for example the Kubernetes cluster registry), not
    /// organization-scoped rights. See
    /// [`crate::authorization::Principal::is_platform_admin`].
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
    ) -> Result<Option<User>, fabrique::Error> {
        User::query()
            .select()
            .r#where(User::EMAIL, "=", email.to_owned())
            .first(pool)
            .await
    }

    pub async fn find_or_create_one_by_email(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<User, fabrique::Error> {
        let maybe_user = User::find_one_by_email(pool, email).await?;

        match maybe_user {
            Some(user) => Ok(user),
            None => {
                User {
                    id: Uuid::new_v4(),
                    is_admin: false,
                    email: email.to_owned(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
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
        Organization::all(connection).await.map_err(Into::into)
    }

    fn is_platform_admin(&self) -> bool {
        self.is_admin
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
        match User::find_one_by_email(&self.db, &email).await? {
            Some(user) => Ok(user),
            None => self.create(principal, email).await,
        }
    }

    pub async fn create<P: Principal>(&self, _principal: &P, email: String) -> Result<User, Error> {
        User::factory()
            .id(Uuid::new_v4())
            .email(email)
            .is_admin(false)
            .create(&self.db)
            .await
            .map_err(Into::into)
    }
}
