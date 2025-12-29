use crate::{
    Error,
    authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource},
    identity::User,
    resourcemanager::{Organization, Organizations},
};
use fabrique::{Factory, Persistable};
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

#[derive(Debug, Default, Factory, Persistable, Resource)]
pub struct Invitation {
    /// The invitation id
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The organization this invitation refers to
    pub organization_id: Uuid,

    /// The user this invitation refers to
    pub user_id: Uuid,

    /// The invitation state
    #[fabrique(as = "String")]
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
        organization_id: <Organization as Resource>::Id,
        user_id: <User as Resource>::Id,
    ) -> Result<Invitation, Error> {
        self.auth
            .can(principal)
            .perform(Permission::InviteMember)
            .over::<Organization>(&organization_id)
            .await?;

        let organization: Organization = sqlx::query_as!(
            Organization,
            "SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE id = $1",
            organization_id
        )
        .fetch_one(&self.db)
        .await?;

        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(&self.db)
            .await?;

        // Create the invitation and mark it as accepted
        let invitation = Invitation::factory()
            .id(Uuid::new_v4())
            .state(InvitationState::Accepted)
            .organization_id(*organization.id())
            .user_id(*user.id())
            .create(&self.db)
            .await?;

        // Add the user to the organization
        self.organizations.add_user(&organization, &user).await?;

        // Create the relationship
        Relationship::new(&user, Relation::Member, &organization)
            .publish(&self.db)
            .await?;
        Ok(invitation)
    }
}
