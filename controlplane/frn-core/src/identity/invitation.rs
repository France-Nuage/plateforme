use database::{Factory, Persistable, Repository};
use sqlx::{FromRow, Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::{
    Error,
    authorization::{Authorize, Permission, Principal, Resource},
    identity::User,
    resourcemanager::Organization,
};

#[derive(Debug, Default, Factory, FromRow, Repository, Resource)]
pub struct Invitation {
    /// The invitation id
    #[repository(primary)]
    pub id: Uuid,

    /// The organization this invitation refers to
    pub organization_id: Uuid,

    /// The user this invitation refers to
    pub user_id: Uuid,

    /// The invitation state
    #[sqlx(try_from = "String")]
    pub state: InvitationState,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum InvitationState {
    #[default]
    Unspecified,
    Pending,
    Accepted,
    Declined,
    Expired,
}

impl From<String> for InvitationState {
    fn from(value: String) -> Self {
        InvitationState::from_str(&value).expect("could not parse value to invitation state")
    }
}

impl From<InvitationState> for String {
    fn from(value: InvitationState) -> Self {
        value.to_string()
    }
}

#[derive(Clone)]
pub struct Invitations<Auth: Authorize> {
    auth: Auth,
    db: Pool<Postgres>,
}

impl<Auth: Authorize> Invitations<Auth> {
    pub fn new(auth: Auth, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    pub async fn list<P: Principal>(&mut self, principal: &P) -> Result<Vec<Invitation>, Error> {
        self.auth
            .lookup()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await
    }

    pub async fn create<P: Principal>(
        &mut self,
        principal: &P,
        organization_id: <Organization as Resource>::Id,
        user_id: <User as Resource>::Id,
    ) -> Result<Invitation, Error> {
        self.auth
            .can(principal)
            .perform(Permission::InviteMember)
            .over::<Organization>(&organization_id)
            .await?;

        Invitation::factory()
            .state(InvitationState::Accepted)
            .organization_id(organization_id)
            .user_id(user_id)
            .create(&self.db)
            .await
            .map_err(Into::into)
    }
}
