use crate::{
    Error,
    authorization::{Authorize, Permission, Principal, Resource},
    identity::User,
    resourcemanager::{Organization, Organizations},
};
use fabrique::{Factory, Model, Query};
use fake::Dummy;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Model, Resource)]
pub struct Invitation {
    /// The invitation id
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The organization this invitation refers to (slug FK)
    pub organization_slug: String,

    /// The user this invitation refers to
    pub user_id: Uuid,

    /// The invitation state
    #[fabrique(as = "String")]
    pub state: InvitationState,

    /// Creation time of the invitation
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the invitation
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Default, Display, Dummy, EnumString, IntoStaticStr)]
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
pub struct Invitations<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
    organizations: Organizations<A>,
}

impl<A: Authorize> Invitations<A> {
    pub fn new(auth: A, db: Pool<Postgres>, organizations: Organizations<A>) -> Self {
        Self {
            auth,
            db,
            organizations,
        }
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
        organization_slug: String,
        user_id: <User as Resource>::Id,
    ) -> Result<Invitation, Error> {
        self.auth
            .can(principal)
            .perform(Permission::InviteMember)
            .over::<Organization>(&organization_slug)
            .await?;

        let organization = Organization::find(&self.db, organization_slug.clone()).await?;

        let user = User::find(&self.db, user_id).await?;

        let invitation = Invitation::factory()
            .id(Uuid::new_v4())
            .state(InvitationState::Accepted)
            .organization_slug(organization.slug.clone())
            .user_id(*user.id())
            .create(&self.db)
            .await?;

        self.organizations.add_user(&organization, &user).await?;

        Ok(invitation)
    }
}
