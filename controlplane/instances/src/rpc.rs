use crate::{
    problem::Problem,
    service::InstancesService,
    v1::{
        CloneInstanceRequest, CreateInstanceRequest, CreateInstanceResponse, DeleteInstanceRequest,
        DeleteInstanceResponse, Instance, ListInstancesRequest, ListInstancesResponse,
        StartInstanceRequest, StartInstanceResponse, StopInstanceRequest, StopInstanceResponse,
        instances_server::Instances,
    },
};
use auth::{Authorize, IAM, Permission};
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct InstancesRpcService {
    pool: PgPool,
    service: InstancesService,
}

#[tonic::async_trait]
impl Instances for InstancesRpcService {
    #[doc = " CreateInstance provisions a new instance based on the specified configuration."]
    #[doc = " Returns details of the newly created instance or a ProblemDetails on failure."]
    async fn create_instance(
        &self,
        request: tonic::Request<CreateInstanceRequest>,
    ) -> Result<tonic::Response<CreateInstanceResponse>, tonic::Status> {
        let request = request.into_inner();
        let project_id = Uuid::parse_str(&request.project_id)
            .map_err(|_| Problem::MalformedInstanceId(request.project_id.clone()))?;

        let instance = self.service.create(request.into(), project_id).await?;

        Ok(Response::new(CreateInstanceResponse {
            instance: Some(instance.into()),
        }))
    }

    #[doc = " DeleteInstance deletes a given instance."]
    #[doc = " Returns an empty message or a ProblemDetails on failure."]
    async fn delete_instance(
        &self,
        request: tonic::Request<DeleteInstanceRequest>,
    ) -> std::result::Result<tonic::Response<DeleteInstanceResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        self.service.delete(id).await?;
        Ok(Response::new(DeleteInstanceResponse {}))
    }

    #[doc = " CloneInstance provisions a new instance based on a given existing instance."]
    #[doc = " Returns the cloned instance."]
    async fn clone_instance(
        &self,
        request: tonic::Request<CloneInstanceRequest>,
    ) -> std::result::Result<tonic::Response<Instance>, tonic::Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        let instance = self.service.clone(id).await?;
        Ok(Response::new(instance.into()))
    }

    #[doc = " ListInstances retrieves information about all available instances."]
    #[doc = " Returns a collection of instance details including their current status and resource usage."]
    async fn list_instances(
        &self,
        _: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let instances = self.service.list().await?;

        Ok(Response::new(ListInstancesResponse {
            instances: instances.into_iter().map(Into::into).collect(),
        }))
    }

    #[doc = " StartInstance initiates a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn start_instance(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let iam = request
            .extensions()
            .get::<IAM>()
            .ok_or(Status::internal("iam not found"))?
            .clone();

        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;

        let user = iam.user(&self.pool).await?;

        iam.authz
            .can(&user)
            .perform(Permission::Get)
            .on((crate::model::Instance::resource_name(), &id))
            .check()
            .await?;

        self.service.start(id).await?;
        Ok(Response::new(StartInstanceResponse {}))
    }

    #[doc = " StopInstance halts a specific instance identified by its unique ID."]
    #[doc = " Returns a response indicating success or a ProblemDetails on failure."]
    async fn stop_instance(
        &self,
        request: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Problem::MalformedInstanceId(id))?;
        self.service.stop(id).await?;
        Ok(Response::new(StopInstanceResponse {}))
    }
}

impl InstancesRpcService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            service: InstancesService::new(pool.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::Instance,
        v1::{ListInstancesRequest, StartInstanceRequest},
    };
    use auth::model::User;
    use hypervisor_connector_proxmox::mock::{
        WithClusterNextId, WithClusterResourceList, WithTaskStatusReadMock, WithVMCloneMock,
        WithVMDeleteMock, WithVMStatusStartMock, WithVMStatusStopMock,
    };
    use mock_server::MockServer;
    use resources::organizations::Organization;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_instances_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the list_instances procedure
        let server = MockServer::new().await.with_cluster_resource_list();
        let mock_url = server.url();

        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");

        Instance::factory()
            .for_hypervisor_with(move |hypervisor| {
                hypervisor
                    .for_default_datacenter()
                    .organization_id(organization.id)
                    .url(mock_url)
            })
            .for_project_with(move |project| project.organization_id(organization.id))
            .create(&pool)
            .await
            .expect("could not create instance");

        let service = InstancesRpcService::new(pool);

        // Act the call to the list_instances procedure
        let result = service
            .list_instances(Request::new(ListInstancesRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        let response = result.unwrap().into_inner();
        // Check that we have a Success result
        assert_eq!(response.instances.len(), 1);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_clone_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_cluster_next_id()
            .with_cluster_resource_list()
            .with_vm_clone()
            .with_task_status_read();
        let mock_url = server.url();

        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");

        let instance = Instance::factory()
            .for_project_with(move |project| project.organization_id(organization.id))
            .for_hypervisor_with(move |hypervisor| {
                hypervisor
                    .for_default_datacenter()
                    .url(mock_url)
                    .organization_id(organization.id)
            })
            .distant_id(String::from("100"))
            .create(&pool)
            .await
            .expect("could not create instance");

        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(CloneInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.clone_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_delete_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_cluster_resource_list()
            .with_vm_delete()
            .with_task_status_read();

        let mock_url = server.url();

        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");

        let instance = Instance::factory()
            .for_project_with(move |project| project.organization_id(organization.id))
            .for_hypervisor_with(move |hypervisor| {
                hypervisor
                    .for_default_datacenter()
                    .url(mock_url)
                    .organization_id(organization.id)
            })
            .distant_id(String::from("100"))
            .create(&pool)
            .await
            .expect("could not create instance");

        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(DeleteInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.delete_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_start_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_cluster_resource_list()
            .with_task_status_read()
            .with_vm_status_start();
        let mock_url = server.url();

        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");

        let user = User::factory()
            .id(Uuid::new_v4())
            .email("wile.coyote@acme.org".to_owned())
            .organization_id(organization.id)
            .create(&pool)
            .await
            .expect("could not create user");

        let instance = Instance::factory()
            .for_project_with(move |project| project.organization_id(organization.id))
            .for_hypervisor_with(move |hypervisor| {
                hypervisor
                    .for_default_datacenter()
                    .url(mock_url)
                    .organization_id(organization.id)
            })
            .distant_id(String::from("100"))
            .create(&pool)
            .await
            .expect("could not create instance");

        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let mut request = Request::new(StartInstanceRequest {
            id: instance.id.to_string(),
        });
        request
            .extensions_mut()
            .insert(IAM::mock().await.for_user(&user));
        let result = service.start_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_stop_instance_works(pool: sqlx::PgPool) {
        // Arrange a service and a request for the start_instance procedure
        let server = MockServer::new()
            .await
            .with_cluster_resource_list()
            .with_task_status_read()
            .with_vm_status_stop();

        let mock_url = server.url();

        let organization = Organization::factory()
            .create(&pool)
            .await
            .expect("could not create organization");

        let instance = Instance::factory()
            .for_project_with(move |project| project.organization_id(organization.id))
            .for_hypervisor_with(move |hypervisor| {
                hypervisor
                    .for_default_datacenter()
                    .url(mock_url)
                    .organization_id(organization.id)
            })
            .distant_id(String::from("100"))
            .create(&pool)
            .await
            .expect("could not create instance");

        let service = InstancesRpcService::new(pool);

        // Act the call to the start_instance procedure
        let request = Request::new(crate::v1::StopInstanceRequest {
            id: instance.id.to_string(),
        });
        let result = service.stop_instance(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
    }
}
