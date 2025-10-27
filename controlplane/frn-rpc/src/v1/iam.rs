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
            organization_id,
        } = request.into_inner();

        let organization_id =
            Uuid::parse_str(&organization_id).map_err(|_| Error::MalformedId(organization_id))?;

        let user_id = self.users.find_or_create(&principal, email).await?.id;

        let invitation = self
            .invitations
            .clone()
            .create(&principal, organization_id, user_id)
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
