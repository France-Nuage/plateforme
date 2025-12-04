use crate::{
    Error,
    authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource},
    identity::User,
    resourcemanager::{Organization, Organizations},
};
use fabrique::{Factory, Persistable};
use sqlx::{FromRow, Pool, Postgres, Type};
use uuid::Uuid;

#[derive(Debug, Default, Factory, FromRow, Persistable, Resource)]
pub struct Invitation {
    /// The invitation id
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The organization this invitation refers to
    pub organization_id: Uuid,

    /// The user this invitation refers to
    pub user_id: Uuid,

    /// The invitation state
    #[fabrique(r#as = "String")]
    pub state: InvitationState,

    /// Creation time of the project
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time of the project
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Clone, Copy, Type)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationState {
    #[default]
    Unspecified,
    Pending,
    Accepted,
    Declined,
    Expired,
}

fn foo(connection: Pool<Postgres>) {
    sqlx::query_as!(
        Invitation,
        r#"SELECT id, organization_id, user_id, state as "state: InvitationState", created_at, updated_at FROM invitations"#
    ).fetch_all(&connection);
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

        let organization = sqlx::query_as!(
            Organization,
            "SELECT * FROM organizations WHERE id = $1",
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
