use std::collections::BTreeMap;
use std::error::Error as StdError;

use frn_core::authorization::{Relation, Relationship};
use frn_core::managed::ManagedServiceInstanceStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::WorkerContext;
use crate::operations::Operations;
use crate::operations::create_k8s_secret::CreateK8sSecretOp;
use crate::operations::create_namespace::CreateNamespaceOp;
use crate::operations::helm_install::HelmInstallOp;
use crate::operations::update_instance_status::UpdateInstanceStatusOp;
use crate::operations::write_relationships::WriteRelationshipsOp;
use crate::workflows::WorkflowDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployManagedServiceWorkflow {
    pub instance_id: Uuid,
    pub project_id: Uuid,
    pub cluster_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub chart_reference: String,
    pub chart_version: String,
    pub values: Value,
    pub secret_data: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,

    #[serde(default, skip_serializing, skip_deserializing)]
    status: DeployStatus,
}

#[derive(Debug, Default)]
enum DeployStatus {
    #[default]
    CreatingNamespace,
    CreatingSecret,
    InstallingHelm,
    WritingRelationships,
    UpdatingStatus,
    Done,
}

impl DeployManagedServiceWorkflow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_id: Uuid,
        project_id: Uuid,
        cluster_id: Uuid,
        namespace: String,
        release_name: String,
        secret_name: String,
        chart_reference: String,
        chart_version: String,
        values: Value,
        secret_data: BTreeMap<String, String>,
        labels: BTreeMap<String, String>,
    ) -> Self {
        Self {
            instance_id,
            project_id,
            cluster_id,
            namespace,
            release_name,
            secret_name,
            chart_reference,
            chart_version,
            values,
            secret_data,
            labels,
            status: DeployStatus::CreatingNamespace,
        }
    }
}

impl WorkflowDefinition for DeployManagedServiceWorkflow {
    type Error = Box<dyn StdError>;

    async fn next_operations(
        &mut self,
        _ctx: WorkerContext,
    ) -> Result<Vec<Operations>, Self::Error> {
        match self.status {
            DeployStatus::CreatingNamespace => {
                self.status = DeployStatus::CreatingSecret;
                Ok(vec![Operations::CreateNamespace(CreateNamespaceOp {
                    namespace: self.namespace.clone(),
                    labels: self.labels.clone(),
                })])
            }
            DeployStatus::CreatingSecret => {
                self.status = DeployStatus::InstallingHelm;
                Ok(vec![Operations::CreateK8sSecret(CreateK8sSecretOp {
                    namespace: self.namespace.clone(),
                    secret_name: self.secret_name.clone(),
                    data: self.secret_data.clone(),
                })])
            }
            DeployStatus::InstallingHelm => {
                self.status = DeployStatus::WritingRelationships;
                Ok(vec![Operations::HelmInstall(HelmInstallOp {
                    release_name: self.release_name.clone(),
                    namespace: self.namespace.clone(),
                    chart_reference: self.chart_reference.clone(),
                    chart_version: self.chart_version.clone(),
                    values: self.values.clone(),
                })])
            }
            DeployStatus::WritingRelationships => {
                self.status = DeployStatus::UpdatingStatus;
                Ok(vec![Operations::WriteRelationships(WriteRelationshipsOp {
                    relationships: vec![Relationship {
                        subject_type: "project".to_owned(),
                        subject_id: self.project_id.to_string(),
                        relation: Relation::Parent,
                        object_type: "managed_service_instance".to_owned(),
                        object_id: self.instance_id.to_string(),
                    }],
                })])
            }
            DeployStatus::UpdatingStatus => {
                self.status = DeployStatus::Done;
                Ok(vec![Operations::UpdateInstanceStatus(
                    UpdateInstanceStatusOp {
                        instance_id: self.instance_id,
                        new_status: ManagedServiceInstanceStatus::Running.to_string(),
                        previous_status: None,
                    },
                )])
            }
            DeployStatus::Done => Ok(vec![]),
        }
    }

    fn target_cluster_id(&self) -> Option<Uuid> {
        Some(self.cluster_id)
    }

    fn name(&self) -> &str {
        "DeployManagedService"
    }
}
