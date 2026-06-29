use frn_core::identity::IAM;
use frn_core::{authorization::Authorize, resourcemanager::ProjectCreateRequest};
use sqlx::{Pool, Postgres};
use std::time::SystemTime;
use tonic::{Request, Response, Status};

tonic::include_proto!("francenuage.fr.resourcemanager.v1");

/// Convert between model and protobuf types
impl From<frn_core::resourcemanager::Organization> for Organization {
    fn from(org: frn_core::resourcemanager::Organization) -> Self {
        Self {
            slug: org.slug,
            name: org.name,
            created_at: Some(SystemTime::from(org.created_at).into()),
            updated_at: Some(SystemTime::from(org.updated_at).into()),
        }
    }
}

/// Convert between model and protobuf types
impl From<frn_core::resourcemanager::Project> for Project {
    fn from(project: frn_core::resourcemanager::Project) -> Self {
        Self {
            slug: project.slug,
            name: project.name,
            organization_slug: project.organization_slug,
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

        let CreateOrganizationRequest { name, parent_slug } = request.into_inner();

        let organization = self
            .organizations
            .clone()
            .create_organization(&self.pool, &principal, name, parent_slug)
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
            .map_err(Status::from)?;

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
            organization_slug,
        } = request.into_inner();

        let request = ProjectCreateRequest {
            name,
            organization_slug,
        };
        let project = self
            .projects
            .clone()
            .create(&principal, request)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(CreateProjectResponse {
            project: Some(project.into()),
        }))
    }
}
