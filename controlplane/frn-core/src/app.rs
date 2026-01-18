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
    compute::{Hypervisors, Instances, Zones},
    identity::{IAM, Invitations, ServiceAccounts, Users},
    longrunning::Operations,
    resourcemanager::{Organizations, Projects},
};
use auth::OpenID;
use spicedb::SpiceDB;
use sqlx::{PgPool, Pool, Postgres};

/// Application state container holding shared dependencies.
///
/// Generic over `Auth` to support different authorization backends
/// (SpiceDB in production, mock in tests).
#[derive(Clone)]
pub struct App<A: Authorize> {
    pub auth: A,
    pub config: Config,
    pub db: Pool<Postgres>,
    pub iam: IAM,
    pub openid: OpenID,

    // services
    pub hypervisors: Hypervisors<A>,
    pub instances: Instances<A>,
    pub invitations: Invitations<A>,
    pub operations: Operations<A>,
    pub organizations: Organizations<A>,
    pub projects: Projects<A>,
    pub service_accounts: ServiceAccounts<A>,
    pub users: Users<A>,
    pub zones: Zones<A>,
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
        let openid = OpenID::discover(reqwest::Client::new(), &config.oidc_url)
            .await
            .map_err(|err| Error::Other(err.to_string()))?;
        let iam = IAM::new(db.clone(), openid.clone());

        let hypervisors = Hypervisors::new(auth.clone(), db.clone());
        let organizations = Organizations::new(auth.clone(), db.clone());
        let instances = Instances::new(auth.clone(), db.clone());
        let invitations = Invitations::new(auth.clone(), db.clone(), organizations.clone());
        let operations = Operations::new(auth.clone(), db.clone());
        let projects = Projects::new(auth.clone(), db.clone());
        let service_accounts = ServiceAccounts::new(auth.clone(), db.clone());
        let users = Users::new(auth.clone(), db.clone());
        let zones = Zones::new(auth.clone(), db.clone());

        let app = Self {
            auth,
            config,
            db,
            iam,
            openid,
            hypervisors,
            instances,
            invitations,
            operations,
            organizations,
            projects,
            service_accounts,
            users,
            zones,
        };

        Ok(app)
    }

    /// Creates a test application instance with mock authorization.
    ///
    /// Uses the provided database pool and a mock SpiceDB server for testing.
    pub async fn test(db: Pool<Postgres>) -> Result<Self, Error> {
        let auth = SpiceDB::mock().await;
        let config = Config::test();
        let openid = OpenID::mock().await;
        let iam = IAM::new(db.clone(), openid.clone());

        let instances = Instances::new(auth.clone(), db.clone());
        let hypervisors = Hypervisors::new(auth.clone(), db.clone());
        let organizations = Organizations::new(auth.clone(), db.clone());
        let invitations = Invitations::new(auth.clone(), db.clone(), organizations.clone());
        let operations = Operations::new(auth.clone(), db.clone());
        let projects = Projects::new(auth.clone(), db.clone());
        let service_accounts = ServiceAccounts::new(auth.clone(), db.clone());
        let users = Users::new(auth.clone(), db.clone());
        let zones = Zones::new(auth.clone(), db.clone());

        let app = Self {
            auth,
            config,
            db,
            iam,
            openid,
            instances,
            hypervisors,
            invitations,
            operations,
            organizations,
            projects,
            service_accounts,
            users,
            zones,
        };

        Ok(app)
    }
}
