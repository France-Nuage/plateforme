use std::collections::BTreeMap;
use std::error::Error as StdError;

use frn_core::authorization::{Permission, Relation, Relationship, Resource};
use frn_core::managed::{ManagedServiceInstanceStatus, WorkflowPrincipal};
use frn_core::resourcemanager::Project;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::WorkerContext;
use crate::operations::Operations;
use crate::operations::assert_namespace_absent::AssertNamespaceAbsentOp;
use crate::operations::check_permission::CheckPermissionOp;
use crate::operations::create_k8s_secret::CreateK8sSecretOp;
use crate::operations::create_namespace::CreateNamespaceOp;
use crate::operations::helm_install::HelmInstallOp;
use crate::operations::update_instance_status::UpdateInstanceStatusOp;
use crate::operations::write_relationships::WriteRelationshipsOp;
use crate::workflows::WorkflowDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployManagedServiceWorkflow {
    pub instance_id: Uuid,
    pub project_slug: String,
    pub cluster_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub chart_reference: String,
    pub chart_version: String,
    pub values: Value,
    pub secret_data: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub principal: Option<WorkflowPrincipal>,

    #[serde(default, skip_serializing, skip_deserializing)]
    status: DeployStatus,
}

#[derive(Debug, Default)]
enum DeployStatus {
    #[default]
    CheckingPermission,
    AssertingNamespaceAbsent,
    WritingRelationships,
    CreatingNamespace,
    CreatingSecret,
    InstallingHelm,
    UpdatingStatus,
    Done,
}

impl DeployManagedServiceWorkflow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_id: Uuid,
        project_slug: String,
        cluster_id: Uuid,
        namespace: String,
        release_name: String,
        secret_name: String,
        chart_reference: String,
        chart_version: String,
        values: Value,
        secret_data: BTreeMap<String, String>,
        labels: BTreeMap<String, String>,
        principal: Option<WorkflowPrincipal>,
    ) -> Self {
        Self {
            instance_id,
            project_slug,
            cluster_id,
            namespace,
            release_name,
            secret_name,
            chart_reference,
            chart_version,
            values,
            secret_data,
            labels,
            principal,
            status: DeployStatus::CheckingPermission,
        }
    }

    fn assert_namespace_absent_op(&self) -> Operations {
        Operations::AssertNamespaceAbsent(AssertNamespaceAbsentOp {
            namespace: self.namespace.clone(),
        })
    }

    fn parent_relationship_op(&self) -> Operations {
        Operations::WriteRelationships(WriteRelationshipsOp {
            relationships: vec![Relationship {
                subject_type: "project".to_owned(),
                subject_id: self.project_slug.clone(),
                relation: Relation::Parent,
                object_type: "managed_service_instance".to_owned(),
                object_id: self.instance_id.to_string(),
            }],
        })
    }
}

impl WorkflowDefinition for DeployManagedServiceWorkflow {
    type Error = Box<dyn StdError>;

    async fn next_operations(
        &mut self,
        _ctx: WorkerContext,
    ) -> Result<Vec<Operations>, Self::Error> {
        match self.status {
            DeployStatus::CheckingPermission => {
                if let Some(ref principal) = self.principal {
                    self.status = DeployStatus::AssertingNamespaceAbsent;
                    Ok(vec![Operations::CheckPermission(CheckPermissionOp {
                        subject_type: principal.principal_type.clone(),
                        subject_id: principal.principal_id.clone(),
                        permission: Permission::CreateInstance.to_string(),
                        resource_type: Project::RESOURCE_NAME.to_owned(),
                        resource_id: self.project_slug.clone(),
                    })])
                } else {
                    info!("no principal set, skipping defense-in-depth permission check");
                    self.status = DeployStatus::WritingRelationships;
                    Ok(vec![self.assert_namespace_absent_op()])
                }
            }
            DeployStatus::AssertingNamespaceAbsent => {
                self.status = DeployStatus::WritingRelationships;
                Ok(vec![self.assert_namespace_absent_op()])
            }
            DeployStatus::WritingRelationships => {
                self.status = DeployStatus::CreatingNamespace;
                Ok(vec![self.parent_relationship_op()])
            }
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
                self.status = DeployStatus::UpdatingStatus;
                Ok(vec![Operations::HelmInstall(HelmInstallOp {
                    release_name: self.release_name.clone(),
                    namespace: self.namespace.clone(),
                    chart_reference: self.chart_reference.clone(),
                    chart_version: self.chart_version.clone(),
                    values: self.values.clone(),
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
