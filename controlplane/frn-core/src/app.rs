//! Application state container
//!
//! Central `App` struct that encapsulates shared application state including
//! database connection, authorization server, identity management, and domain
//! services. Generic over authorization backend to enable swapping between
//! SpiceDB and mock implementations for testing.
//!
//! Use `App::new()` for production initialization from environment, or
//! `App::test()` with a test database pool for testing with mock authorization.

use crate::{
    Config, Error, authorization::AuthorizationServer, identity::IAM,
    resourcemanager::OrganizationService,
};
use spicedb::SpiceDB;
use sqlx::{PgPool, Pool, Postgres};

/// Application state container holding shared dependencies.
///
/// Generic over `Auth` to support different authorization backends
/// (SpiceDB in production, mock in tests).
#[derive(Clone)]
pub struct App<Auth: AuthorizationServer> {
    pub auth: Auth,
    pub db: Pool<Postgres>,
    pub iam: IAM,

    // services
    pub organizations: OrganizationService<Auth>,
}

impl App<SpiceDB> {
    /// Creates a new production application instance.
    ///
    /// Initializes from environment variables, connects to SpiceDB and PostgreSQL,
    /// and composes all services. Returns an error if configuration is missing or
    /// connections fail.
    pub async fn new() -> Result<Self, Error> {
        let config = Config::from_env()?;
        let auth = SpiceDB::connect(&config.auth_server_url, &config.auth_server_token).await?;
        let db = PgPool::connect(&config.database_url).await?;
        let iam = IAM::new();

        let organizations = OrganizationService::new(auth.clone(), db.clone());

        let app = Self {
            auth,
            db,
            iam,
            organizations,
        };

        Ok(app)
    }

    /// Creates a test application instance with mock authorization.
    ///
    /// Uses the provided database pool and a mock SpiceDB server for testing.
    pub async fn test(db: Pool<Postgres>) -> Result<Self, Error> {
        let auth = SpiceDB::mock().await;
        let iam = IAM::new();
        let organizations = OrganizationService::new(auth.clone(), db.clone());

        let app = Self {
            auth,
            db,
            iam,
            organizations,
        };

        Ok(app)
    }
}
