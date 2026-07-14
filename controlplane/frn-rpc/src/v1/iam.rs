use std::time::SystemTime;

use frn_core::authorization::{Authorize, Principal as _, Resource as _};
use frn_core::identity::{IAM, Principal};
use tonic::{Request, Response, Status};

tonic::include_proto!("francenuage.fr.v1.iam");

impl From<frn_core::identity::InvitationState> for InvitationState {
    fn from(value: frn_core::identity::InvitationState) -> Self {
        match value {
            frn_core::identity::InvitationState::Unspecified => InvitationState::Unspecified,
            frn_core::identity::InvitationState::Pending => InvitationState::Pending,
            frn_core::identity::InvitationState::Accepted => InvitationState::Accepted,
            frn_core::identity::InvitationState::Declined => InvitationState::Declined,
            frn_core::identity::InvitationState::Expired => InvitationState::Expired,
        }
    }
}

impl From<frn_core::identity::Invitation> for Invitation {
    fn from(value: frn_core::identity::Invitation) -> Self {
        Invitation {
            id: value.id.to_string(),
            organization_slug: value.organization_slug.clone(),
            user_id: value.user_id.to_string(),
            state: InvitationState::from(value.state) as i32,
            created_at: Some(SystemTime::from(value.created_at).into()),
            answered_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}

pub struct Invitations<Auth: Authorize> {
    iam: IAM,
    invitations: frn_core::identity::Invitations<Auth>,
    users: frn_core::identity::Users<Auth>,
}

impl<Auth: Authorize> Invitations<Auth> {
    pub fn new(
        iam: IAM,
        invitations: frn_core::identity::Invitations<Auth>,
        users: frn_core::identity::Users<Auth>,
    ) -> Self {
        Self {
            iam,
            invitations,
            users,
        }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> invitations_server::Invitations for Invitations<Auth> {
    async fn list(
        &self,
        request: Request<ListInvitationsRequest>,
    ) -> Result<Response<ListInvitationsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let invitations = self.invitations.clone().list(&principal).await?;

        Ok(Response::new(ListInvitationsResponse {
            invitations: invitations.into_iter().map(Into::into).collect(),
        }))
    }

    async fn create(
        &self,
        request: Request<CreateInvitationRequest>,
    ) -> Result<Response<CreateInvitationResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let CreateInvitationRequest {
            email,
            organization_slug,
        } = request.into_inner();

        let user_id = self.users.find_or_create(&principal, email).await?.id;

        let invitation = self
            .invitations
            .clone()
            .create(&principal, organization_slug, user_id)
            .await?;

        Ok(Response::new(CreateInvitationResponse {
            invitation: Some(invitation.into()),
        }))
    }

    async fn answer(
        &self,
        _: Request<AnswerInvitationRequest>,
    ) -> Result<Response<AnswerInvitationResponse>, Status> {
        unimplemented!()
    }
}

/// Service exposing the authenticated caller's own identity.
///
/// `GetCurrentUser` lets the frontend read its platform-admin status from the
/// authoritative source (the control plane database, via the resolved
/// principal) instead of decoding it from the OIDC token. Keycloak
/// authenticates the caller; the control plane decides what it can do.
pub struct Profile {
    iam: IAM,
}

impl Profile {
    pub fn new(iam: IAM) -> Self {
        Self { iam }
    }
}

#[tonic::async_trait]
impl profile_server::Profile for Profile {
    /// Returns the calling principal's id, email and platform-admin flag.
    ///
    /// The principal is resolved from the Bearer token and read straight from
    /// the database, so `is_admin` reflects `users.is_admin` at request time.
    /// Service accounts have no email and are never platform admins.
    async fn get_current_user(
        &self,
        request: Request<GetCurrentUserRequest>,
    ) -> Result<Response<GetCurrentUserResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let id = principal.id().to_string();
        let is_admin = principal.is_platform_admin();
        let email = match &principal {
            Principal::User(user) => user.email.clone(),
            Principal::ServiceAccount(_) => String::new(),
        };

        Ok(Response::new(GetCurrentUserResponse {
            id,
            email,
            is_admin,
        }))
    }
}
