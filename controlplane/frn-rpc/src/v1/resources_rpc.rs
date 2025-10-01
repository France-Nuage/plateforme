use sqlx::{Pool, Postgres, types::Uuid};
use tonic::{Request, Response, Status};

use crate::{
    error::Error,
    v1::resources::{
        CreateOrganizationRequest, CreateOrganizationResponse, CreateProjectRequest,
        CreateProjectResponse, ListProjectsRequest, ListProjectsResponse,
        resources_server::Resources,
    },
};

/// Implementation of the Resources gRPC service.
///
/// This service handles operations related to resource management,
/// including listing and creating organizations and projects. It uses a database
/// connection to persist and retrieve resource information.
pub struct ResourcesRpcService {
    /// The database pool
    pool: Pool<Postgres>,
}

impl ResourcesRpcService {
    /// Creates a new instance of the Resources gRPC service.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection for resource data persistence
    ///
    /// # Returns
    ///
    /// A new `ResourcesRpcService` instance
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl Resources for ResourcesRpcService {
    #[doc = " ListOrganizations retrieves information about all available organizations."]
    #[doc = " Returns a collection of organizations."]
    async fn list_organizations(
        &self,
        request: tonic::Request<super::resources::ListOrganizationsRequest>,
    ) -> std::result::Result<
        tonic::Response<super::resources::ListOrganizationsResponse>,
        tonic::Status,
    > {
        let iam = request
            .extensions()
            .get::<auth::IAM>()
            .ok_or(tonic::Status::internal("iam not found"))?;

        let user = iam.user(&self.pool).await?;

        let organizations = frn_core::services::organization::list(&self.pool, user)
            .await
            .map_err(Error::convert)?;

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
        let CreateOrganizationRequest { name } = request.into_inner();

        let organization = frn_core::services::organization::create(&self.pool, name)
            .await
            .map_err(Error::convert)?;

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
        let projects = frn_core::services::project::list(&self.pool)
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

        let project = frn_core::services::project::create(&self.pool, name, organization_id)
            .await
            .map_err(Error::convert)?;

        Ok(Response::new(CreateProjectResponse {
            project: Some(project.into()),
        }))
    }
}
