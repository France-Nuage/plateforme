use std::collections::BTreeMap;
use std::error::Error as StdError;

use frn_core::authorization::{Relation, Relationship};
use frn_core::managed::ManagedServiceInstanceStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::WorkerContext;
use crate::operations::Operations;
use crate::operations::delete_k8s_secret::DeleteK8sSecretOp;
use crate::operations::delete_namespace::DeleteNamespaceOp;
use crate::operations::delete_relationships::DeleteRelationshipsOp;
use crate::operations::helm_uninstall::HelmUninstallOp;
use crate::operations::update_instance_status::UpdateInstanceStatusOp;
use crate::workflows::WorkflowDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteManagedServiceWorkflow {
    pub instance_id: Uuid,
    pub project_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub chart_reference: String,
    pub chart_version: String,
    pub values: Value,
    pub labels: BTreeMap<String, String>,

    #[serde(default, skip_serializing, skip_deserializing)]
    status: DeleteStatus,
}

#[derive(Debug, Default)]
enum DeleteStatus {
    #[default]
    UninstallingHelm,
    DeletingSecret,
    DeletingNamespace,
    DeletingRelationships,
    MarkingDeleted,
    Done,
}

impl DeleteManagedServiceWorkflow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_id: Uuid,
        project_id: Uuid,
        namespace: String,
        release_name: String,
        secret_name: String,
        chart_reference: String,
        chart_version: String,
        values: Value,
        labels: BTreeMap<String, String>,
    ) -> Self {
        Self {
            instance_id,
            project_id,
            namespace,
            release_name,
            secret_name,
            chart_reference,
            chart_version,
            values,
            labels,
            status: DeleteStatus::UninstallingHelm,
        }
    }
}

impl WorkflowDefinition for DeleteManagedServiceWorkflow {
    type Error = Box<dyn StdError>;

    async fn next_operations(
        &mut self,
        _ctx: WorkerContext,
    ) -> Result<Vec<Operations>, Self::Error> {
        match self.status {
            DeleteStatus::UninstallingHelm => {
                self.status = DeleteStatus::DeletingSecret;
                Ok(vec![Operations::HelmUninstall(HelmUninstallOp {
                    release_name: self.release_name.clone(),
                    namespace: self.namespace.clone(),
                    chart_reference: self.chart_reference.clone(),
                    chart_version: self.chart_version.clone(),
                    values: self.values.clone(),
                })])
            }
            DeleteStatus::DeletingSecret => {
                self.status = DeleteStatus::DeletingNamespace;
                Ok(vec![Operations::DeleteK8sSecret(DeleteK8sSecretOp {
                    namespace: self.namespace.clone(),
                    secret_name: self.secret_name.clone(),
                    data: BTreeMap::new(),
                })])
            }
            DeleteStatus::DeletingNamespace => {
                self.status = DeleteStatus::DeletingRelationships;
                Ok(vec![Operations::DeleteNamespace(DeleteNamespaceOp {
                    namespace: self.namespace.clone(),
                    labels: self.labels.clone(),
                })])
            }
            DeleteStatus::DeletingRelationships => {
                self.status = DeleteStatus::MarkingDeleted;
                Ok(vec![Operations::DeleteRelationships(
                    DeleteRelationshipsOp {
                        relationships: vec![Relationship {
                            subject_type: "project".to_owned(),
                            subject_id: self.project_id.to_string(),
                            relation: Relation::Parent,
                            object_type: "managed_service_instance".to_owned(),
                            object_id: self.instance_id.to_string(),
                        }],
                    },
                )])
            }
            DeleteStatus::MarkingDeleted => {
                self.status = DeleteStatus::Done;
                Ok(vec![Operations::UpdateInstanceStatus(
                    UpdateInstanceStatusOp {
                        instance_id: self.instance_id,
                        new_status: ManagedServiceInstanceStatus::Deleted.to_string(),
                        previous_status: None,
                    },
                )])
            }
            DeleteStatus::Done => Ok(vec![]),
        }
    }

    fn name(&self) -> &str {
        "DeleteManagedService"
    }
}
