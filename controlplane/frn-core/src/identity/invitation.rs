use crate::{
    Error,
    authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource},
    identity::User,
    operations::{Operation, OperationType, TargetBackend},
    resourcemanager::{Organization, Organizations},
};
use chrono::{DateTime, Duration, Utc};
use fabrique::{Factory, Persistable};
use rand::Rng;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

/// Default validity duration for invitations in hours.
const DEFAULT_INVITATION_VALIDITY_HOURS: i64 = 168; // 7 days

#[derive(Debug, Default, Factory, Persistable, Resource)]
pub struct Invitation {
    /// The invitation id
    #[fabrique(primary_key)]
    pub id: Uuid,

    /// The organization this invitation refers to
    pub organization_id: Uuid,

    /// The user this invitation refers to (optional for email-based invitations)
    pub user_id: Option<Uuid>,

    /// The email address the invitation was sent to (for email-based invitations)
    pub email: Option<String>,

    /// The role ID to assign when the invitation is accepted
    pub role_id: Option<Uuid>,

    /// Secure token for invitation URLs
    pub token: Option<String>,

    /// The invitation state
    #[fabrique(as = "String")]
    pub state: InvitationState,

    /// When the invitation expires
    pub expires_at: Option<DateTime<Utc>>,

    /// When the invitation was answered (accepted or declined)
    pub answered_at: Option<DateTime<Utc>>,

    /// Creation time of the invitation
    pub created_at: DateTime<Utc>,

    /// Last update time of the invitation
    pub updated_at: DateTime<Utc>,
}

impl Invitation {
    /// Generates a secure random token for invitation URLs.
    pub fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.r#gen();
        hex::encode(bytes)
    }

    /// Returns true if the invitation has expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expires| Utc::now() > expires)
            .unwrap_or(false)
    }

    /// Returns true if the invitation can be answered (pending and not expired).
    pub fn can_answer(&self) -> bool {
        matches!(self.state, InvitationState::Pending) && !self.is_expired()
    }
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

/// Helper function to map a database row to an Invitation struct.
fn row_to_invitation(row: sqlx::postgres::PgRow) -> Invitation {
    use sqlx::Row;
    Invitation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        user_id: row.get("user_id"),
        email: row.get("email"),
        role_id: row.get("role_id"),
        token: row.get("token"),
        state: row.get::<String, _>("state").into(),
        expires_at: row.get("expires_at"),
        answered_at: row.get("answered_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
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

    /// Creates an invitation by email address.
    ///
    /// This creates the invitation in PENDING state and creates an Operation
    /// to sync with Pangolin. The user may not exist yet - they'll be linked
    /// when they accept the invitation.
    ///
    /// # Returns
    /// A tuple of (Invitation, Operation) where the Operation is for Pangolin sync.
    pub async fn create_by_email<P: Principal>(
        &mut self,
        principal: &P,
        organization_id: <Organization as Resource>::Id,
        email: &str,
        role_id: Option<Uuid>,
        validity_hours: Option<i64>,
    ) -> Result<(Invitation, Operation), Error> {
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

        let token = Invitation::generate_token();
        let hours = validity_hours.unwrap_or(DEFAULT_INVITATION_VALIDITY_HOURS);
        let expires_at = Utc::now() + Duration::hours(hours);
        let invitation_id = Uuid::new_v4();

        // Create the invitation in PENDING state
        let invitation = Invitation::factory()
            .id(invitation_id)
            .state(InvitationState::Pending)
            .organization_id(*organization.id())
            .email(Some(email.to_string()))
            .role_id(role_id)
            .token(Some(token))
            .expires_at(Some(expires_at))
            .create(&self.db)
            .await?;

        // Create Operation for Pangolin sync
        let role_id_str = role_id.map(|id| id.to_string()).unwrap_or_default();
        let operation = Operation::factory()
            .operation_type(OperationType::PangolinInviteUser)
            .target_backend(TargetBackend::Pangolin)
            .resource_type("Invitation".to_string())
            .resource_id(invitation_id)
            .input(serde_json::json!({
                "org_id": organization.slug,
                "email": email,
                "role_id": role_id_str,
                "send_email": true,
                "valid_for_hours": hours
            }))
            .create(&self.db)
            .await?;

        Ok((invitation, operation))
    }

    /// Creates an invitation for an existing user.
    ///
    /// # Returns
    /// A tuple of (Invitation, Operation) where the Operation is for Pangolin sync.
    pub async fn create<P: Principal>(
        &mut self,
        principal: &P,
        organization_id: <Organization as Resource>::Id,
        user_id: <User as Resource>::Id,
        role_id: Option<Uuid>,
    ) -> Result<(Invitation, Operation), Error> {
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

        let token = Invitation::generate_token();
        let expires_at = Utc::now() + Duration::hours(DEFAULT_INVITATION_VALIDITY_HOURS);
        let invitation_id = Uuid::new_v4();

        // Create the invitation in PENDING state
        let invitation = Invitation::factory()
            .id(invitation_id)
            .state(InvitationState::Pending)
            .organization_id(*organization.id())
            .user_id(Some(*user.id()))
            .email(Some(user.email.clone()))
            .role_id(role_id)
            .token(Some(token))
            .expires_at(Some(expires_at))
            .create(&self.db)
            .await?;

        // Create Operation for Pangolin sync
        let role_id_str = role_id.map(|id| id.to_string()).unwrap_or_default();
        let operation = Operation::factory()
            .operation_type(OperationType::PangolinInviteUser)
            .target_backend(TargetBackend::Pangolin)
            .resource_type("Invitation".to_string())
            .resource_id(invitation_id)
            .input(serde_json::json!({
                "org_id": organization.slug,
                "email": user.email,
                "role_id": role_id_str,
                "send_email": true,
                "valid_for_hours": DEFAULT_INVITATION_VALIDITY_HOURS
            }))
            .create(&self.db)
            .await?;

        Ok((invitation, operation))
    }

    /// Retrieves an invitation by its token.
    pub async fn get_by_token(&self, token: &str) -> Result<Option<Invitation>, Error> {
        let row = sqlx::query(
            r#"SELECT id, organization_id, user_id, email, role_id, token,
                      state, expires_at, answered_at, created_at, updated_at
               FROM invitations WHERE token = $1"#,
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(row_to_invitation))
    }

    /// Answers an invitation (accept or decline).
    ///
    /// If accepted:
    /// - Links the user to the invitation if not already linked
    /// - Adds the user to the organization
    /// - Creates a SpiceDB sync operation for the membership relationship
    ///
    /// # Arguments
    /// * `invitation_id` - The invitation ID
    /// * `user` - The user answering the invitation
    /// * `accept` - Whether to accept (true) or decline (false)
    ///
    /// # Returns
    /// A tuple of (Invitation, Option<Operation>) where the Operation is for SpiceDB sync
    /// (only present when accepting).
    pub async fn answer(
        &mut self,
        invitation_id: Uuid,
        user: &User,
        accept: bool,
    ) -> Result<(Invitation, Option<Operation>), Error> {
        // Fetch the invitation
        let invitation = sqlx::query(
            r#"SELECT id, organization_id, user_id, email, role_id, token,
                      state, expires_at, answered_at, created_at, updated_at
               FROM invitations WHERE id = $1"#,
        )
        .bind(invitation_id)
        .fetch_one(&self.db)
        .await
        .map(row_to_invitation)?;

        // Verify the invitation can be answered
        if !invitation.can_answer() {
            if invitation.is_expired() {
                return Err(Error::InvalidArgument("invitation has expired".to_string()));
            }
            return Err(Error::InvalidArgument(
                "invitation has already been answered".to_string(),
            ));
        }

        // Verify the user matches the invitation (if user_id is set)
        if let Some(inv_user_id) = invitation.user_id {
            if inv_user_id != *user.id() {
                return Err(Error::PermissionDenied(
                    "user does not match invitation".to_string(),
                ));
            }
        }

        // Verify email matches (if email is set and no user_id)
        if invitation.user_id.is_none() {
            if let Some(ref inv_email) = invitation.email {
                if inv_email != &user.email {
                    return Err(Error::PermissionDenied(
                        "user email does not match invitation".to_string(),
                    ));
                }
            }
        }

        let new_state = if accept {
            InvitationState::Accepted
        } else {
            InvitationState::Declined
        };

        // Update the invitation
        sqlx::query(
            r#"UPDATE invitations
               SET state = $2, user_id = $3, answered_at = now(), updated_at = now()
               WHERE id = $1"#,
        )
        .bind(invitation_id)
        .bind(new_state.to_string())
        .bind(*user.id())
        .execute(&self.db)
        .await?;

        // Fetch the updated invitation
        let updated_invitation = sqlx::query(
            r#"SELECT id, organization_id, user_id, email, role_id, token,
                      state, expires_at, answered_at, created_at, updated_at
               FROM invitations WHERE id = $1"#,
        )
        .bind(invitation_id)
        .fetch_one(&self.db)
        .await
        .map(row_to_invitation)?;

        if !accept {
            return Ok((updated_invitation, None));
        }

        // Fetch the organization
        let organization: Organization = sqlx::query_as!(
            Organization,
            "SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE id = $1",
            invitation.organization_id
        )
        .fetch_one(&self.db)
        .await?;

        // Add the user to the organization locally
        self.organizations.add_user(&organization, user).await?;

        // Create SpiceDB sync operation for the membership
        let operation = Operation::factory()
            .operation_type(OperationType::SpiceDbWriteRelationship)
            .target_backend(TargetBackend::SpiceDb)
            .resource_type("User".to_string())
            .resource_id(*user.id())
            .input(serde_json::json!({
                "subject_type": "User",
                "subject_id": user.id().to_string(),
                "relation": "Member",
                "object_type": "Organization",
                "object_id": organization.id().to_string()
            }))
            .create(&self.db)
            .await?;

        // Also publish the relationship via the existing mechanism
        Relationship::new(user, Relation::Member, &organization)
            .publish(&self.db)
            .await?;

        Ok((updated_invitation, Some(operation)))
    }
}
