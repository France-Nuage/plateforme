use crate::{Config, Error, authorization::AuthorizationServer, identity::IAM};
use spicedb::SpiceDB;
use sqlx::{PgPool, Pool, Postgres};

pub struct App<Auth: AuthorizationServer> {
    pub auth: Auth,
    pub db: Pool<Postgres>,
    pub iam: IAM,
}

impl App<SpiceDB> {
    pub async fn new() -> Result<Self, Error> {
        let config = Config::from_env();
        let auth = SpiceDB::connect(&config.auth_server_url, &config.auth_server_token).await?;
        let db = PgPool::connect(&config.database_url).await?;
        let iam = IAM::new();

        let app = Self { auth, db, iam };

        Ok(app)
    }
}
