use crate::{
    Config, Error, authorization::AuthorizationServer, identity::IAM,
    resourcemanager::OrganizationService,
};
use spicedb::SpiceDB;
use sqlx::{PgPool, Pool, Postgres};

#[derive(Clone)]
pub struct App<Auth: AuthorizationServer> {
    pub auth: Auth,
    pub db: Pool<Postgres>,
    pub iam: IAM,

    // services
    pub organizations: OrganizationService<Auth>,
}

impl App<SpiceDB> {
    pub async fn new() -> Result<Self, Error> {
        let config = Config::from_env();
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
