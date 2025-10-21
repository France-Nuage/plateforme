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
    Config, Error,
    authorization::Authorize,
    compute::{Hypervisors, Zones},
    identity::IAM,
    resourcemanager::{Organizations, Projects},
};
use spicedb::SpiceDB;
use sqlx::{PgPool, Pool, Postgres};

/// Application state container holding shared dependencies.
///
/// Generic over `Auth` to support different authorization backends
/// (SpiceDB in production, mock in tests).
#[derive(Clone)]
pub struct App<Auth: Authorize> {
    pub auth: Auth,
    pub db: Pool<Postgres>,
    pub iam: IAM,

    // services
    pub hypervisors: Hypervisors<Auth>,
    pub organizations: Organizations<Auth>,
    pub projects: Projects<Auth>,
    pub zones: Zones<Auth>,
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
        let iam = IAM::new(db.clone());

        let hypervisors = Hypervisors::new(auth.clone(), db.clone());
        let organizations = Organizations::new(auth.clone(), db.clone());
        let projects = Projects::new(auth.clone(), db.clone());
        let zones = Zones::new(auth.clone(), db.clone());

        let app = Self {
            auth,
            db,
            iam,
            hypervisors,
            organizations,
            projects,
            zones,
        };

        Ok(app)
    }

    /// Creates a test application instance with mock authorization.
    ///
    /// Uses the provided database pool and a mock SpiceDB server for testing.
    pub async fn test(db: Pool<Postgres>) -> Result<Self, Error> {
        let auth = SpiceDB::mock().await;
        let iam = IAM::new(db.clone());

        let hypervisors = Hypervisors::new(auth.clone(), db.clone());
        let organizations = Organizations::new(auth.clone(), db.clone());
        let projects = Projects::new(auth.clone(), db.clone());
        let zones = Zones::new(auth.clone(), db.clone());

        let app = Self {
            auth,
            db,
            iam,
            hypervisors,
            organizations,
            projects,
            zones,
        };

        Ok(app)
    }
}
