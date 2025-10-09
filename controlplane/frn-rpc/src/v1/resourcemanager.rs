use crate::error::Error;
use crate::request::ExtractToken;
use frn_core::authorization::AuthorizationServer;
use frn_core::identity::IAM;
use sqlx::{Pool, Postgres, types::Uuid};
use std::{marker::PhantomData, time::SystemTime};
use tonic::{Request, Response, Status};

tonic::include_proto!("francenuage.fr.resourcemanager.v1");

/// Convert between model and protobuf types
impl From<frn_core::resourcemanager::Organization> for Organization {
    fn from(org: frn_core::resourcemanager::Organization) -> Self {
        Self {
            id: org.id.to_string(),
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
            id: project.id.to_string(),
            name: project.name,
            organization_id: project.organization_id.to_string(),
            created_at: Some(SystemTime::from(project.created_at).into()),
            updated_at: Some(SystemTime::from(project.updated_at).into()),
        }
    }
}

pub struct Organizations<Auth: AuthorizationServer> {
    iam: IAM,
    pool: Pool<Postgres>,
    organizations: frn_core::resourcemanager::Organizations<Auth>,
}

impl<Auth: AuthorizationServer> Organizations<Auth> {
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
impl<Auth: AuthorizationServer + 'static> organizations_server::Organizations
    for Organizations<Auth>
{
    async fn list(
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
            super::resourcemanager::ListOrganizationsResponse {
                organizations: organizations.into_iter().map(Into::into).collect(),
            },
        ))
    }

    async fn create(
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
            super::resourcemanager::CreateOrganizationResponse {
                organization: Some(organization.into()),
            },
        ))
    }
}

pub struct Projects<Auth: AuthorizationServer> {
    _auth: PhantomData<Auth>,
    _iam: IAM,
    pool: Pool<Postgres>,
}

impl<Auth: AuthorizationServer> Projects<Auth> {
    pub fn new(iam: IAM, pool: Pool<Postgres>) -> Self {
        Self {
            _iam: iam,
            pool,
            _auth: PhantomData,
        }
    }
}

#[tonic::async_trait]
impl<Auth: AuthorizationServer + 'static> projects_server::Projects for Projects<Auth> {
    async fn list(
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

    async fn create(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
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
