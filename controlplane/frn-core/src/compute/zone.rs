use crate::Error;
use crate::authorization::{AuthorizationServer, Principal, Resource};
use chrono::{DateTime, Utc};
use database::{Factory, Persistable, Repository};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, PartialEq, Repository, Resource)]
pub struct Zone {
    /// Unique identifier for the zone
    #[repository(primary)]
    pub id: Uuid,

    /// A human-readable name for the zone
    pub name: String,

    // Creation time of the zone
    pub created_at: DateTime<Utc>,

    // Time of the datancenter last update
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Zones<Auth: AuthorizationServer> {
    _auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: AuthorizationServer> Zones<Auth> {
    /// Creates a new zones service.
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { _auth: auth, db }
    }

    /// Lists all zones accessible to the principal
    pub async fn list<P: Principal>(&mut self, _principal: &P) -> Result<Vec<Zone>, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::List)
        //     .over(&Zone::any())
        //     .await?;

        Zone::list(&self.db).await.map_err(Into::into)
    }
}
