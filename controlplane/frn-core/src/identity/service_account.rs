use crate::Error;
use crate::authorization::{Authorize, Principal, Resource};
use crate::resourcemanager::Organization;
use chrono::{DateTime, Utc};
use fabrique::{Factory, Persistable};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

/// Non-human identity for programmatic access using API keys
#[derive(Debug, Default, Factory, FromRow, Persistable, Resource)]
#[fabrique(table = "service_accounts")]
pub struct ServiceAccount {
    #[fabrique(primary_key)]
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
        Organization::all(connection).await.map_err(Into::into)
    }
}

pub struct ServiceAccountCreateRequest {}

#[derive(Clone)]
pub struct ServiceAccounts<Auth: Authorize> {
    _auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> ServiceAccounts<Auth> {
    /// Creates a new service accounts service.
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { _auth: auth, db }
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

    pub async fn initialize_root_service_account(
        &self,
        organization: &Organization,
        name: String,
        key: Option<String>,
    ) -> Result<ServiceAccount, Error> {
        let maybe_service_account = sqlx::query_as!(
            ServiceAccount,
            "
            SELECT service_accounts.id, service_accounts.name, service_accounts.key, service_accounts.created_at, service_accounts.updated_at
            FROM service_accounts
            JOIN organization_service_account ON organization_service_account.service_account_id = service_accounts.id
            JOIN organizations ON organization_service_account.organization_id = organizations.id
            WHERE organizations.id = $1
            AND service_accounts.name = $2
            LIMIT 1
            ",
            organization.id,
            name
        ).fetch_optional(&self.db).await?;

        let service_account = match maybe_service_account {
            Some(service_account) => service_account,
            None => {
                let key = key.ok_or(Error::MissingEnvVar("ROOT_SERVICE_ACCOUNT_KEY".to_owned()))?;
                ServiceAccount::factory()
                    .name(name)
                    .key(key)
                    .create(&self.db)
                    .await?
            }
        };

        Ok(service_account)
    }
}
