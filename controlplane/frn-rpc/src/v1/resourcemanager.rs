use crate::error::Error;
use frn_core::identity::{IAM, Users};
use frn_core::{authorization::Authorize, resourcemanager::ProjectCreateRequest};
use sqlx::{Pool, Postgres, types::Uuid};
use std::time::SystemTime;
use tonic::{Request, Response, Status};

tonic::include_proto!("francenuage.fr.resourcemanager.v1");

/// Convert between model and protobuf types for OrganizationMember
impl From<frn_core::resourcemanager::OrganizationMember> for OrganizationMember {
    fn from(member: frn_core::resourcemanager::OrganizationMember) -> Self {
        Self {
            user_id: member.user_id.to_string(),
            organization_id: member.organization_id.to_string(),
            email: member.email,
            name: member.name,
            role_id: member.role_id.map(|id| id.to_string()).unwrap_or_default(),
            joined_at: Some(SystemTime::from(member.joined_at).into()),
        }
    }
}

/// Convert between model and protobuf types
impl From<frn_core::resourcemanager::Organization> for Organization {
    fn from(org: frn_core::resourcemanager::Organization) -> Self {
        Self {
            id: org.id.to_string(),
            name: org.name,
            slug: org.slug,
            created_at: Some(SystemTime::from(org.created_at).into()),
            updated_at: Some(SystemTime::from(org.updated_at).into()),
        }
    }
}

/// Convert between model and protobuf types
impl From<frn_core::resourcemanager::Project> for Project {
    fn from(project: frn_core::resourcemanager::Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            organization_id: project.organization_id.to_string(),
            created_at: Some(SystemTime::from(project.created_at).into()),
            updated_at: Some(SystemTime::from(project.updated_at).into()),
        }
    }
}

pub struct Organizations<Auth: Authorize> {
    iam: IAM,
    pool: Pool<Postgres>,
    organizations: frn_core::resourcemanager::Organizations<Auth>,
}

impl<Auth: Authorize> Organizations<Auth> {
    pub fn new(
        iam: IAM,
        organizations: frn_core::resourcemanager::Organizations<Auth>,
        pool: Pool<Postgres>,
    ) -> Self {
        Self {
            iam,
            organizations,
            pool,
        }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> organizations_server::Organizations for Organizations<Auth> {
    async fn list(
        &self,
        request: tonic::Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let organizations = self.organizations.clone().list(&principal).await?;

        Ok(tonic::Response::new(
            super::resourcemanager::ListOrganizationsResponse {
                organizations: organizations.into_iter().map(Into::into).collect(),
            },
        ))
    }

    async fn create(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let CreateOrganizationRequest { name, parent_id } = request.into_inner();

        let parent_id = parent_id
            .map(|value| Uuid::parse_str(&value).map_err(|_| Status::invalid_argument("")))
            .transpose()?;

        let organization = self
            .organizations
            .clone()
            .create_organization(&self.pool, &principal, name, parent_id)
            .await?;

        Ok(tonic::Response::new(
            super::resourcemanager::CreateOrganizationResponse {
                organization: Some(organization.into()),
            },
        ))
    }
}

pub struct Projects<A: Authorize> {
    iam: IAM,
    projects: frn_core::resourcemanager::Projects<A>,
}

impl<Auth: Authorize> Projects<Auth> {
    pub fn new(iam: IAM, projects: frn_core::resourcemanager::Projects<Auth>) -> Self {
        Self { iam, projects }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> projects_server::Projects for Projects<Auth> {
    async fn list(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let projects = self
            .projects
            .clone()
            .list(&principal)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(ListProjectsResponse {
            projects: projects.into_iter().map(Into::into).collect(),
        }))
    }

    async fn create(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let CreateProjectRequest {
            name,
            organization_id,
        } = request.into_inner();
        let organization_id = Uuid::parse_str(&organization_id)
            .map_err(|_| Status::invalid_argument("invalid argument organization_id"))?;

        let request = ProjectCreateRequest {
            name,
            organization_id,
        };
        let project = self
            .projects
            .clone()
            .create(&principal, request)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(CreateProjectResponse {
            project: Some(project.into()),
        }))
    }
}

/// OrganizationMembers service manages user membership within organizations.
pub struct OrganizationMembers<Auth: Authorize> {
    iam: IAM,
    db: Pool<Postgres>,
    organizations: frn_core::resourcemanager::Organizations<Auth>,
    _users: Users<Auth>,
}

impl<Auth: Authorize> OrganizationMembers<Auth> {
    pub fn new(
        iam: IAM,
        db: Pool<Postgres>,
        organizations: frn_core::resourcemanager::Organizations<Auth>,
        users: Users<Auth>,
    ) -> Self {
        Self {
            iam,
            db,
            organizations,
            _users: users,
        }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> organization_members_server::OrganizationMembers
    for OrganizationMembers<Auth>
{
    async fn list(
        &self,
        request: Request<ListOrganizationMembersRequest>,
    ) -> Result<Response<ListOrganizationMembersResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let ListOrganizationMembersRequest { organization_id } = request.into_inner();

        let organization_id = Uuid::parse_str(&organization_id)
            .map_err(|_| Error::MalformedId(organization_id))?;

        let members = self
            .organizations
            .clone()
            .list_members(&principal, organization_id)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(ListOrganizationMembersResponse {
            members: members.into_iter().map(Into::into).collect(),
        }))
    }

    async fn remove(
        &self,
        request: Request<RemoveOrganizationMemberRequest>,
    ) -> Result<Response<RemoveOrganizationMemberResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let RemoveOrganizationMemberRequest {
            organization_id,
            user_id,
        } = request.into_inner();

        let organization_id =
            Uuid::parse_str(&organization_id).map_err(|_| Error::MalformedId(organization_id))?;
        let user_id = Uuid::parse_str(&user_id).map_err(|_| Error::MalformedId(user_id))?;

        // Fetch the organization
        let organization: frn_core::resourcemanager::Organization = sqlx::query_as!(
            frn_core::resourcemanager::Organization,
            "SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE id = $1",
            organization_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(Error::sqlx_to_status)?;

        // Fetch the user
        let user: frn_core::identity::User =
            sqlx::query_as!(frn_core::identity::User, "SELECT * FROM users WHERE id = $1", user_id)
                .fetch_one(&self.db)
                .await
                .map_err(Error::sqlx_to_status)?;

        // Remove the user from the organization - this creates operations for sync
        let operations = self
            .organizations
            .clone()
            .remove_user(&principal, &organization, &user)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(RemoveOrganizationMemberResponse {
            operation_names: operations.into_iter().map(|op| op.name).collect(),
        }))
    }
}
