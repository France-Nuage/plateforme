use crate::Error;
use crate::authorization::{Authorize, Principal, Resource};
use chrono::{DateTime, Utc};
use fabrique::{Factory, Model, Query};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, PartialEq, Resource)]
pub struct Zone {
    /// Unique identifier for the zone
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// A human-readable name for the zone
    pub name: String,

    // Creation time of the zone
    pub created_at: DateTime<Utc>,

    // Time of the datancenter last update
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Zones<Auth: Authorize> {
    _auth: Auth,
    db: Pool<Postgres>,
}

pub struct ZoneCreateRequest {
    pub name: String,
}

impl<Auth: Authorize> Zones<Auth> {
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

        Zone::all(&self.db).await.map_err(Into::into)
    }

    pub async fn create<P: Principal>(
        &mut self,
        _principal: &P,
        request: ZoneCreateRequest,
    ) -> Result<Zone, Error> {
        Zone::factory()
            .id(Uuid::new_v4())
            .name(request.name)
            .create(&self.db)
            .await
            .map_err(Into::into)
    }
}
