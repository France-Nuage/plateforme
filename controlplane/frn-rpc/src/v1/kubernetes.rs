use std::collections::HashMap;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::error::Error;
use crate::timestamp::to_timestamp;
use frn_core::identity::IAM;
use frn_core::kubernetes::{
    CreateClusterInput, KubernetesCluster, KubernetesClusterError, KubernetesClusters,
    KubernetesLabel, KubernetesLabelError, KubernetesLabels, UpdateClusterInput,
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
    labels: KubernetesLabels,
}

impl KubernetesClustersRpc {
    pub fn new(iam: IAM, service: KubernetesClusters, labels: KubernetesLabels) -> Self {
        Self {
            iam,
            service,
            labels,
        }
    }

    async fn hydrate_cluster_proto(
        &self,
        principal: &impl frn_core::authorization::Principal,
        cluster: &KubernetesCluster,
    ) -> Result<KubernetesClusterProto, Status> {
        let mut proto = KubernetesClusterProto::from(cluster);
        proto.labels = self
            .labels
            .list_cluster_labels(principal, cluster.id)
            .await
            .map_err(kubernetes_label_error_to_status)?
            .iter()
            .map(Into::into)
            .collect();
        Ok(proto)
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
            kubernetes_version: cluster.kubernetes_version.clone(),
            platform: cluster.platform.clone(),
            health_status: cluster.health_status.to_string(),
            last_health_check_at: cluster.last_health_check_at.map(to_timestamp),
            created_at: Some(to_timestamp(cluster.created_at)),
            updated_at: Some(to_timestamp(cluster.updated_at)),
            labels: Vec::new(),
        }
    }
}

impl From<&KubernetesLabel> for KubernetesLabelProto {
    fn from(label: &KubernetesLabel) -> Self {
        Self {
            id: label.id.to_string(),
            key: label.key.clone(),
            value: label.value.clone(),
            system: label.system,
            created_at: Some(to_timestamp(label.created_at)),
            updated_at: Some(to_timestamp(label.updated_at)),
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
        KubernetesClusterError::ClusterHasInstances(_) => Status::failed_precondition(message),
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

fn kubernetes_label_error_to_status(err: KubernetesLabelError) -> Status {
    let message = err.to_string();
    match err {
        KubernetesLabelError::Forbidden => Status::permission_denied(message),
        KubernetesLabelError::NotFound(_) | KubernetesLabelError::ClusterNotFound(_) => {
            Status::not_found(message)
        }
        KubernetesLabelError::AlreadyExists { .. } => Status::already_exists(message),
        KubernetesLabelError::InvalidKey(_) | KubernetesLabelError::InvalidValue(_) => {
            Status::invalid_argument(message)
        }
        KubernetesLabelError::SystemLabelReadOnly(_) => Status::failed_precondition(message),
        KubernetesLabelError::Database(_) | KubernetesLabelError::Fabrique(_) => {
            tracing::error!(error = %message, "internal kubernetes label error");
            Status::internal("internal error")
        }
    }
}

fn parse_id(raw: &str) -> Result<Uuid, Error> {
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

        // One query hydrates every cluster's labels (no per-cluster
        // round-trip).
        let mut labels_by_cluster: HashMap<Uuid, Vec<KubernetesLabelProto>> = HashMap::new();
        for (cluster_id, label) in self
            .labels
            .list_all_cluster_labels(&principal)
            .await
            .map_err(kubernetes_label_error_to_status)?
        {
            labels_by_cluster
                .entry(cluster_id)
                .or_default()
                .push((&label).into());
        }

        Ok(Response::new(ListClustersResponse {
            clusters: clusters
                .iter()
                .map(|cluster| {
                    let mut proto = KubernetesClusterProto::from(cluster);
                    proto.labels = labels_by_cluster.remove(&cluster.id).unwrap_or_default();
                    proto
                })
                .collect(),
        }))
    }

    async fn get_cluster(
        &self,
        request: Request<GetClusterRequest>,
    ) -> Result<Response<GetClusterResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let cluster_id = parse_id(&request.into_inner().cluster_id)?;

        let cluster = self
            .service
            .get_cluster(&principal, cluster_id)
            .await
            .map_err(kubernetes_error_to_status)?;

        let proto = self.hydrate_cluster_proto(&principal, &cluster).await?;

        Ok(Response::new(GetClusterResponse {
            cluster: Some(proto),
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

        let cluster_id = parse_id(&req.cluster_id)?;
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

        let proto = self.hydrate_cluster_proto(&principal, &cluster).await?;

        Ok(Response::new(UpdateClusterResponse {
            cluster: Some(proto),
        }))
    }

    async fn delete_cluster(
        &self,
        request: Request<DeleteClusterRequest>,
    ) -> Result<Response<DeleteClusterResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let cluster_id = parse_id(&request.into_inner().cluster_id)?;

        self.service
            .delete_cluster(&principal, cluster_id)
            .await
            .map_err(kubernetes_error_to_status)?;

        Ok(Response::new(DeleteClusterResponse {}))
    }

    async fn list_labels(
        &self,
        request: Request<ListLabelsRequest>,
    ) -> Result<Response<ListLabelsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let labels = self
            .labels
            .list_labels(&principal)
            .await
            .map_err(kubernetes_label_error_to_status)?;

        Ok(Response::new(ListLabelsResponse {
            labels: labels.iter().map(Into::into).collect(),
        }))
    }

    async fn create_label(
        &self,
        request: Request<CreateLabelRequest>,
    ) -> Result<Response<CreateLabelResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        if req.key.trim().is_empty() {
            return Err(Error::InvalidInput("key is required".to_owned()))?;
        }
        if req.value.trim().is_empty() {
            return Err(Error::InvalidInput("value is required".to_owned()))?;
        }

        let label = self
            .labels
            .create_label(&principal, req.key, req.value)
            .await
            .map_err(kubernetes_label_error_to_status)?;

        Ok(Response::new(CreateLabelResponse {
            label: Some((&label).into()),
        }))
    }

    async fn delete_label(
        &self,
        request: Request<DeleteLabelRequest>,
    ) -> Result<Response<DeleteLabelResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let label_id = parse_id(&request.into_inner().label_id)?;

        self.labels
            .delete_label(&principal, label_id)
            .await
            .map_err(kubernetes_label_error_to_status)?;

        Ok(Response::new(DeleteLabelResponse {}))
    }

    async fn attach_cluster_label(
        &self,
        request: Request<AttachClusterLabelRequest>,
    ) -> Result<Response<AttachClusterLabelResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();
        let cluster_id = parse_id(&req.cluster_id)?;
        let label_id = parse_id(&req.label_id)?;

        self.labels
            .attach_label(&principal, cluster_id, label_id)
            .await
            .map_err(kubernetes_label_error_to_status)?;

        Ok(Response::new(AttachClusterLabelResponse {}))
    }

    async fn detach_cluster_label(
        &self,
        request: Request<DetachClusterLabelRequest>,
    ) -> Result<Response<DetachClusterLabelResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();
        let cluster_id = parse_id(&req.cluster_id)?;
        let label_id = parse_id(&req.label_id)?;

        self.labels
            .detach_label(&principal, cluster_id, label_id)
            .await
            .map_err(kubernetes_label_error_to_status)?;

        Ok(Response::new(DetachClusterLabelResponse {}))
    }
}
