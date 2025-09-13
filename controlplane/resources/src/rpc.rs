//! Implementation of the gRPC service for resources management.
//!
//! This module provides the implementation of the Resources gRPC service,
//! handling requests to manage organizations and projects.

use crate::{
    organizations::{Organization, OrganizationService},
    projects::{Project, ProjectService},
    v1::{
        CreateOrganizationRequest, CreateOrganizationResponse, CreateProjectRequest,
        CreateProjectResponse, ListOrganizationsRequest, ListOrganizationsResponse,
        ListProjectsRequest, ListProjectsResponse, resources_server::Resources,
    },
};
use auth::IAM;
use sqlx::{PgPool, Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// Implementation of the Resources gRPC service.
///
/// This service handles operations related to resource management,
/// including listing and creating organizations and projects. It uses a database
/// connection to persist and retrieve resource information.
pub struct ResourcesRpcService {
    /// The database pool
    pool: Pool<Postgres>,

    /// The project service.
    project_service: ProjectService,

    /// The oranization service.
    organization_service: OrganizationService,
}

#[tonic::async_trait]
impl Resources for ResourcesRpcService {
    /// Lists all organizations.
    ///
    /// This method retrieves all organization records from the database
    /// and returns them as a collection.
    ///
    /// # Arguments
    ///
    /// * `_` - Empty request
    ///
    /// # Returns
    ///
    /// * `Ok(Response<ListOrganizationsResponse>)` - Contains the list of organizations
    /// * `Err(Status)` - If retrieval fails, with appropriate status code
    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        let user = request
            .extensions()
            .get::<IAM>()
            .ok_or(Status::internal("iam not found"))?
            .user(&self.pool)
            .await?;

        let organizations = Organization::find_by_user(&self.pool, user).await?;

        Ok(Response::new(ListOrganizationsResponse {
            organizations: organizations.into_iter().map(Into::into).collect(),
        }))
    }

    /// Creates a new organization.
    ///
    /// This method persists the organization information provided in the request
    /// to the database, generating a new UUID for the organization.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the organization details to be created
    ///
    /// # Returns
    ///
    /// * `Ok(Response<CreateOrganizationResponse>)` - On successful creation
    /// * `Err(Status)` - If creation fails, with appropriate status code
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
        let organization_request = request.into_inner();

        let organization = Organization {
            id: Uuid::new_v4(),
            name: organization_request.name,
            ..Default::default()
        };

        let organization = self.organization_service.create(organization).await?;

        Ok(Response::new(CreateOrganizationResponse {
            organization: Some(organization.into()),
        }))
    }

    /// Lists all projects, optionally filtered by organization.
    ///
    /// This method retrieves project records from the database
    /// and returns them as a collection.
    /// # Arguments
    ///
    /// * `_` - Empty request
    ///
    /// # Returns
    ///
    /// * `Ok(Response<ListProjectsResponse>)` - Contains the list of projects
    /// * `Err(Status)` - If retrieval fails, with appropriate status code
    async fn list_projects(
        &self,
        _: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let projects = self.project_service.list().await?;

        Ok(Response::new(ListProjectsResponse {
            projects: projects.into_iter().map(Into::into).collect(),
        }))
    }

    /// Creates a new project.
    ///
    /// This method persists the project information provided in the request
    /// to the database, generating a new UUID for the project.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the project details to be created
    ///
    /// # Returns
    ///
    /// * `Ok(Response<CreateProjectResponse>)` - On successful creation
    /// * `Err(Status)` - If creation fails, with appropriate status code
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        let project_request = request.into_inner();

        let organization_id = Uuid::parse_str(&project_request.organization_id)
            .map_err(|_| Status::invalid_argument("Invalid organization_id format"))?;

        let project = Project {
            id: Uuid::new_v4(),
            name: project_request.name,
            organization_id,
            ..Default::default()
        };

        let project = self.project_service.create(project).await?;

        Ok(Response::new(CreateProjectResponse {
            project: Some(project.into()),
        }))
    }
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
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            project_service: ProjectService::new(pool.clone()),
            organization_service: OrganizationService::new(pool.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::{
        CreateOrganizationRequest, CreateProjectRequest, ListOrganizationsRequest,
        ListProjectsRequest,
    };
    use auth::{
        OpenID,
        mock::{WithJwks, WithWellKnown},
        model::User,
    };
    use mock_server::MockServer;
    use sqlx::PgPool;
    use tonic::Request;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_organizations_works(pool: PgPool) {
        // Arrange a service with necessary dependencies
        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");
        let user = User::factory()
            .organization_id(organization.id)
            .email("wile.coyote@acme.org".to_owned())
            .is_admin(true)
            .create(&pool)
            .await
            .expect("could not create user");
        let mock = MockServer::new().await.with_well_known().with_jwks();
        let token = OpenID::token(&user.email);
        let jwk = OpenID::discover(
            reqwest::Client::new(),
            &format!("{}/.well-known/openid-configuration", &mock.url()),
        )
        .await
        .unwrap();
        let iam = IAM::new(Some(format!("Bearer {}", token)), jwk);
        let service = ResourcesRpcService::new(pool);

        // Act the call to the list_organizations procedure
        let mut request = Request::new(ListOrganizationsRequest::default());
        request.extensions_mut().insert(iam);

        let result = service.list_organizations(request).await;

        // Assert the procedure result
        println!("result: {:#?}", &result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_inner().organizations.len(), 1);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_create_organization_works(pool: PgPool) {
        // Arrange a service
        let service = ResourcesRpcService::new(pool);

        // Act the call to the create_organization procedure
        let request = Request::new(CreateOrganizationRequest {
            name: "Test Organization".to_string(),
        });
        let result = service.create_organization(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        assert!(response.organization.is_some());
        assert_eq!(response.organization.unwrap().name, "Test Organization");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_projects_works(pool: PgPool) {
        // Arrange a service
        let service = ResourcesRpcService::new(pool);

        // Act the call to the list_projects procedure
        let result = service
            .list_projects(Request::new(ListProjectsRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        assert_eq!(response.projects.len(), 0);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_create_project_works(pool: PgPool) {
        // Arrange a service and an organization
        let service = ResourcesRpcService::new(pool.clone());

        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");

        // Act the call to the create_project procedure
        let request = Request::new(CreateProjectRequest {
            name: "Test Project".to_string(),
            organization_id: organization.id.to_string(),
        });
        let result = service.create_project(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        assert!(response.project.is_some());
        let project = response.project.unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.organization_id, organization.id.to_string());
    }
}
