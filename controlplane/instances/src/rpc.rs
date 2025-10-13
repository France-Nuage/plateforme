use crate::{
    error::Error,
    service::InstancesService,
    v1::{
        CloneInstanceRequest, CreateInstanceRequest, CreateInstanceResponse, DeleteInstanceRequest,
        DeleteInstanceResponse, Instance, ListInstancesRequest, ListInstancesResponse,
        StartInstanceRequest, StartInstanceResponse, StopInstanceRequest, StopInstanceResponse,
        instances_server::Instances,
    },
};
use frn_core::{
    authorization::AuthorizationServer, compute::Hypervisors, identity::IAM,
    resourcemanager::Projects,
};
use frn_rpc::request::ExtractToken;
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct InstancesRpcService<Auth: AuthorizationServer> {
    iam: IAM,
    service: InstancesService<Auth>,
}

#[tonic::async_trait]
impl<Auth: AuthorizationServer + 'static> Instances for InstancesRpcService<Auth> {
    /// CreateInstance provisions a new instance based on the specified configuration.
    /// Returns details of the newly created instance or a ProblemDetails on failure.
    async fn create_instance(
        &self,
        request: tonic::Request<CreateInstanceRequest>,
    ) -> Result<tonic::Response<CreateInstanceResponse>, tonic::Status> {
        let principal = self.iam.user(request.access_token()).await?;

        let request = request.into_inner();
        let project_id = Uuid::parse_str(&request.project_id)
            .map_err(|_| Error::MalformedInstanceId(request.project_id.clone()))?;

        let instance = self
            .service
            .clone()
            .create(request.into(), project_id, &principal)
            .await?;

        Ok(Response::new(CreateInstanceResponse {
            instance: Some(instance.into()),
        }))
    }

    /// DeleteInstance deletes a given instance.
    /// Returns an empty message or a ProblemDetails on failure.
    async fn delete_instance(
        &self,
        request: tonic::Request<DeleteInstanceRequest>,
    ) -> std::result::Result<tonic::Response<DeleteInstanceResponse>, tonic::Status> {
        let principal = self.iam.user(request.access_token()).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedInstanceId(id))?;
        self.service.clone().delete(&principal, id).await?;
        Ok(Response::new(DeleteInstanceResponse {}))
    }

    /// CloneInstance provisions a new instance based on a given existing instance.
    /// Returns the cloned instance.
    async fn clone_instance(
        &self,
        request: tonic::Request<CloneInstanceRequest>,
    ) -> std::result::Result<tonic::Response<Instance>, tonic::Status> {
        let principal = self.iam.user(request.access_token()).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedInstanceId(id))?;

        let instance = self.service.clone().clone_instance(id, &principal).await?;

        Ok(Response::new(instance.into()))
    }

    /// ListInstances retrieves information about all available instances.
    /// Returns a collection of instance details including their current status and resource usage.
    async fn list_instances(
        &self,
        _: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let instances = self.service.list().await?;

        Ok(Response::new(ListInstancesResponse {
            instances: instances.into_iter().map(Into::into).collect(),
        }))
    }

    /// StartInstance initiates a specific instance identified by its unique ID.
    /// Returns a response indicating success or a ProblemDetails on failure.
    async fn start_instance(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedInstanceId(id))?;

        self.service.start(&principal, id).await?;
        Ok(Response::new(StartInstanceResponse {}))
    }

    /// StopInstance halts a specific instance identified by its unique ID.
    /// Returns a response indicating success or a ProblemDetails on failure.
    async fn stop_instance(
        &self,
        request: Request<StopInstanceRequest>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        let principal = self.iam.user(request.access_token()).await?;
        let id = request.into_inner().id;
        let id = Uuid::parse_str(&id).map_err(|_| Error::MalformedInstanceId(id))?;
        self.service.stop(&principal, id).await?;
        Ok(Response::new(StopInstanceResponse {}))
    }
}

impl<Auth: AuthorizationServer> InstancesRpcService<Auth> {
    pub fn new(
        iam: IAM,
        pool: PgPool,
        hypervisors: Hypervisors<Auth>,
        projects: Projects<Auth>,
    ) -> Self {
        Self {
            iam,
            service: InstancesService::new(pool.clone(), hypervisors, projects),
        }
    }
}
