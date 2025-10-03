use crate::error::Error;
use crate::request::ExtractToken;
use crate::v1::resources::{
    CreateOrganizationRequest, CreateOrganizationResponse, CreateProjectRequest,
    CreateProjectResponse, ListOrganizationsRequest, ListOrganizationsResponse,
    ListProjectsRequest, ListProjectsResponse, resources_server::Resources,
};
use frn_core::{
    authorization::AuthorizationServer, identity::IAM, resourcemanager::OrganizationService,
};
use sqlx::{Pool, Postgres, types::Uuid};
use tonic::{Request, Response, Status};

/// Implementation of the Resources gRPC service.
///
/// This service handles operations related to resource management,
/// including listing and creating organizations and projects. It uses a database
/// connection to persist and retrieve resource information.
pub struct ResourcesRpcService<Auth: AuthorizationServer> {
    iam: IAM,
    pool: Pool<Postgres>,
    organizations: OrganizationService<Auth>,
}

impl<Auth: AuthorizationServer> ResourcesRpcService<Auth> {
    /// Creates a new instance of the Resources gRPC service.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection for resource data persistence
    ///
    /// # Returns
    ///
    /// A new `ResourcesRpcService` instance
    pub fn new(iam: IAM, organizations: OrganizationService<Auth>, pool: Pool<Postgres>) -> Self {
        Self {
            iam,
            organizations,
            pool,
        }
    }
}

#[tonic::async_trait]
impl<Auth: AuthorizationServer + Sync + 'static> Resources for ResourcesRpcService<Auth> {
    #[doc = " ListOrganizations retrieves information about all available organizations."]
    #[doc = " Returns a collection of organizations."]
    async fn list_organizations(
        &self,
        request: tonic::Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;

        let organizations = self
            .organizations
            .clone()
            .list_organizations(&principal)
            .await?;

        Ok(tonic::Response::new(
            super::resources::ListOrganizationsResponse {
                organizations: organizations.into_iter().map(Into::into).collect(),
            },
        ))
    }

    #[doc = " CreateOrganization creates a new organization with the specified name."]
    #[doc = " Returns the newly created organization."]
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;

        let CreateOrganizationRequest { name } = request.into_inner();

        let organization = self
            .organizations
            .clone()
            .create_organization(&self.pool, &principal, name)
            .await?;

        Ok(tonic::Response::new(
            super::resources::CreateOrganizationResponse {
                organization: Some(organization.into()),
            },
        ))
    }

    #[doc = " ListProjects retrieves information about all available projects for a specific organization."]
    #[doc = " Returns a collection of projects."]
    async fn list_projects(
        &self,
        _request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let projects = frn_core::resourcemanager::list_projects(&self.pool)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(ListProjectsResponse {
            projects: projects.into_iter().map(Into::into).collect(),
        }))
    }

    #[doc = " CreateProject creates a new project with the specified name within an organization."]
    #[doc = " Returns the newly created project."]
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, tonic::Status> {
        let CreateProjectRequest {
            name,
            organization_id,
        } = request.into_inner();
        let organization_id = Uuid::parse_str(&organization_id)
            .map_err(|_| Status::invalid_argument("invalid argument organization_id"))?;

        let project = frn_core::resourcemanager::create_project(&self.pool, name, organization_id)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(CreateProjectResponse {
            project: Some(project.into()),
        }))
    }
}
