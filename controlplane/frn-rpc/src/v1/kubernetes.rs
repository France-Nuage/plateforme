use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::error::Error;
use crate::timestamp::to_timestamp;
use frn_core::identity::IAM;
use frn_core::kubernetes::{
    CreateClusterInput, KubernetesCluster, KubernetesClusterError, KubernetesClusters,
    UpdateClusterInput,
};

tonic::include_proto!("francenuage.fr.v1.kubernetes");

/// gRPC transport for the platform-admin Kubernetes cluster registry.
///
/// Resolves the calling principal via [`IAM`] and delegates to the service
/// layer, which enforces the platform-admin requirement. HTTP concerns only:
/// input parsing, validation, and error mapping.
pub struct KubernetesClustersRpc {
    iam: IAM,
    service: KubernetesClusters,
}

impl KubernetesClustersRpc {
    pub fn new(iam: IAM, service: KubernetesClusters) -> Self {
        Self { iam, service }
    }
}

impl From<&KubernetesCluster> for KubernetesClusterProto {
    fn from(cluster: &KubernetesCluster) -> Self {
        Self {
            id: cluster.id.to_string(),
            name: cluster.name.clone(),
            description: cluster.description.clone(),
            api_server_url: cluster.api_server_url.clone(),
            ca_fingerprint: cluster.ca_fingerprint.clone(),
            health_status: cluster.health_status.to_string(),
            last_health_check_at: cluster.last_health_check_at.map(to_timestamp),
            created_at: Some(to_timestamp(cluster.created_at)),
            updated_at: Some(to_timestamp(cluster.updated_at)),
        }
    }
}

fn kubernetes_error_to_status(err: KubernetesClusterError) -> Status {
    let message = err.to_string();
    match err {
        KubernetesClusterError::Forbidden => Status::permission_denied(message),
        KubernetesClusterError::NotFound(_) => Status::not_found(message),
        KubernetesClusterError::NameAlreadyExists(_) => Status::already_exists(message),
        KubernetesClusterError::InvalidName(_) => Status::invalid_argument(message),
        KubernetesClusterError::HealthCheck(_) => Status::failed_precondition(message),
        KubernetesClusterError::ClusterHasProjects(_) => Status::failed_precondition(message),
        KubernetesClusterError::Database(_)
        | KubernetesClusterError::Fabrique(_)
        | KubernetesClusterError::Encryption(_)
        | KubernetesClusterError::InvalidUtf8
        | KubernetesClusterError::InvalidKubeconfig(_)
        | KubernetesClusterError::KubeClientBuild(_) => {
            tracing::error!(error = %message, "internal kubernetes cluster error");
            Status::internal("internal error")
        }
    }
}

fn parse_cluster_id(raw: &str) -> Result<Uuid, Error> {
    raw.parse::<Uuid>()
        .map_err(|_| Error::MalformedId(raw.to_owned()))
}

#[tonic::async_trait]
impl kubernetes_clusters_server::KubernetesClusters for KubernetesClustersRpc {
    async fn list_clusters(
        &self,
        request: Request<ListClustersRequest>,
    ) -> Result<Response<ListClustersResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let clusters = self
            .service
            .list_clusters(&principal)
            .await
            .map_err(kubernetes_error_to_status)?;

        Ok(Response::new(ListClustersResponse {
            clusters: clusters.iter().map(Into::into).collect(),
        }))
    }

    async fn get_cluster(
        &self,
        request: Request<GetClusterRequest>,
    ) -> Result<Response<GetClusterResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let cluster_id = parse_cluster_id(&request.into_inner().cluster_id)?;

        let cluster = self
            .service
            .get_cluster(&principal, cluster_id)
            .await
            .map_err(kubernetes_error_to_status)?;

        Ok(Response::new(GetClusterResponse {
            cluster: Some((&cluster).into()),
        }))
    }

    async fn create_cluster(
        &self,
        request: Request<CreateClusterRequest>,
    ) -> Result<Response<CreateClusterResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        if req.name.trim().is_empty() {
            return Err(Error::InvalidInput("name is required".to_owned()))?;
        }
        if req.kubeconfig.trim().is_empty() {
            return Err(Error::InvalidInput("kubeconfig is required".to_owned()))?;
        }

        let cluster = self
            .service
            .create_cluster(
                &principal,
                CreateClusterInput {
                    name: req.name,
                    description: req.description,
                    kubeconfig: req.kubeconfig,
                },
            )
            .await
            .map_err(kubernetes_error_to_status)?;

        Ok(Response::new(CreateClusterResponse {
            cluster: Some((&cluster).into()),
        }))
    }

    async fn update_cluster(
        &self,
        request: Request<UpdateClusterRequest>,
    ) -> Result<Response<UpdateClusterResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        let cluster_id = parse_cluster_id(&req.cluster_id)?;
        if req.name.trim().is_empty() {
            return Err(Error::InvalidInput("name is required".to_owned()))?;
        }
        if let Some(kubeconfig) = &req.kubeconfig
            && kubeconfig.trim().is_empty()
        {
            return Err(Error::InvalidInput(
                "kubeconfig must not be empty when provided".to_owned(),
            ))?;
        }

        let cluster = self
            .service
            .update_cluster(
                &principal,
                cluster_id,
                UpdateClusterInput {
                    name: req.name,
                    description: req.description,
                    kubeconfig: req.kubeconfig,
                },
            )
            .await
            .map_err(kubernetes_error_to_status)?;

        Ok(Response::new(UpdateClusterResponse {
            cluster: Some((&cluster).into()),
        }))
    }

    async fn delete_cluster(
        &self,
        request: Request<DeleteClusterRequest>,
    ) -> Result<Response<DeleteClusterResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let cluster_id = parse_cluster_id(&request.into_inner().cluster_id)?;

        self.service
            .delete_cluster(&principal, cluster_id)
            .await
            .map_err(kubernetes_error_to_status)?;

        Ok(Response::new(DeleteClusterResponse {}))
    }
}
