use crate::Error;
use crate::authorization::{Authorize, Principal, Resource};
use crate::resourcemanager::Organization;
use chrono::{DateTime, Utc};
use database::{Factory, Persistable, Repository};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

/// Non-human identity for programmatic access using API keys
#[derive(Debug, Default, Factory, FromRow, Repository, Resource)]
pub struct ServiceAccount {
    #[repository(primary)]
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// API key for authentication
    pub key: String,

    /// Creation time of the instance
    pub created_at: DateTime<Utc>,

    /// Time of the instance last update
    pub updated_at: DateTime<Utc>,
}

impl Principal for ServiceAccount {
    /// Returns all organizations this service account has access to
    async fn organizations(
        &self,
        connection: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<crate::resourcemanager::Organization>, crate::Error> {
        Organization::list(connection).await.map_err(Into::into)
    }
}

pub struct ServiceAccountCreateRequest {}

pub struct ServiceAccounts<Auth: Authorize> {
    _auth: Auth,
    _db: Pool<Postgres>,
}

impl<Auth: Authorize> ServiceAccounts<Auth> {
    /// Creates a new service accounts service.
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self {
            _auth: auth,
            _db: db,
        }
    }
    /// Lists all service accounts accessible to the principal.
    pub async fn list<P: Principal>(
        &mut self,
        _principal: &P,
    ) -> Result<Vec<ServiceAccount>, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::List)
        //     .over(&ServiceAccount::any())
        //     .check()
        //     .await?;
        todo!()
    }

    /// Creates a new service account.
    pub async fn create<P: Principal>(
        &mut self,
        _principal: &P,
        _request: ServiceAccountCreateRequest,
    ) -> Result<ServiceAccount, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&ServiceAccount::any())
        //     .check()
        //     .await?;
        todo!()
    }
}
