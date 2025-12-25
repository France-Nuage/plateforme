use std::time::SystemTime;

use frn_core::{authorization::Authorize, identity::IAM};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::error::Error;

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
            organization_id: value.organization_id.to_string(),
            user_id: value.user_id.map(|id| id.to_string()).unwrap_or_default(),
            state: InvitationState::from(value.state) as i32,
            email: value.email.unwrap_or_default(),
            role_id: value.role_id.map(|id| id.to_string()).unwrap_or_default(),
            token: value.token.unwrap_or_default(),
            created_at: Some(SystemTime::from(value.created_at).into()),
            expires_at: value.expires_at.map(|t| SystemTime::from(t).into()),
            answered_at: value.answered_at.map(|t| SystemTime::from(t).into()),
        }
    }
}

pub struct Invitations<Auth: Authorize> {
    iam: IAM,
    invitations: frn_core::identity::Invitations<Auth>,
    _users: frn_core::identity::Users<Auth>,
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
            _users: users,
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
            organization_id,
            role_id,
            validity_hours,
        } = request.into_inner();

        let organization_id =
            Uuid::parse_str(&organization_id).map_err(|_| Error::MalformedId(organization_id))?;

        // Parse optional role_id
        let role_id = role_id
            .filter(|s| !s.is_empty())
            .map(|s| Uuid::parse_str(&s).map_err(|_| Error::MalformedId(s)))
            .transpose()?;

        // Create the invitation by email - this returns the invitation and an operation
        let (invitation, _operation) = self
            .invitations
            .clone()
            .create_by_email(&principal, organization_id, &email, role_id, validity_hours)
            .await?;

        Ok(Response::new(CreateInvitationResponse {
            invitation: Some(invitation.into()),
        }))
    }

    async fn answer(
        &self,
        request: Request<AnswerInvitationRequest>,
    ) -> Result<Response<AnswerInvitationResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let AnswerInvitationRequest {
            invitation_id,
            accept,
        } = request.into_inner();

        let invitation_id =
            Uuid::parse_str(&invitation_id).map_err(|_| Error::MalformedId(invitation_id))?;

        // Extract user from principal - only users can answer invitations
        let user = match principal {
            frn_core::identity::Principal::User(user) => user,
            frn_core::identity::Principal::ServiceAccount(_) => {
                return Err(Status::permission_denied(
                    "only users can answer invitations",
                ));
            }
        };

        // Answer the invitation - this returns the invitation and optionally an operation
        let (invitation, _operation) = self
            .invitations
            .clone()
            .answer(invitation_id, &user, accept)
            .await?;

        Ok(Response::new(AnswerInvitationResponse {
            invitation: Some(invitation.into()),
        }))
    }
}
