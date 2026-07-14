use std::collections::BTreeMap;
use std::error::Error as StdError;

use frn_core::managed::ManagedServiceInstanceStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::WorkerContext;
use crate::operations::Operations;
use crate::operations::helm_upgrade::HelmUpgradeOp;
use crate::operations::update_instance_status::UpdateInstanceStatusOp;
use crate::operations::update_instance_version::UpdateInstanceVersionOp;
use crate::operations::update_k8s_secret::UpdateK8sSecretOp;
use crate::workflows::WorkflowDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeManagedServiceWorkflow {
    pub instance_id: Uuid,
    pub cluster_id: Uuid,
    pub version_id: Uuid,
    pub namespace: String,
    pub release_name: String,
    pub secret_name: String,
    pub chart_reference: String,
    pub chart_version: String,
    pub values: Value,
    pub secret_data: BTreeMap<String, String>,

    #[serde(default, skip_serializing, skip_deserializing)]
    status: UpgradeStatus,
}

#[derive(Debug, Default)]
enum UpgradeStatus {
    #[default]
    UpdatingSecret,
    UpgradingHelm,
    UpdatingVersion,
    MarkingRunning,
    Done,
}

impl UpgradeManagedServiceWorkflow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_id: Uuid,
        cluster_id: Uuid,
        version_id: Uuid,
        namespace: String,
        release_name: String,
        secret_name: String,
        chart_reference: String,
        chart_version: String,
        values: Value,
        secret_data: BTreeMap<String, String>,
    ) -> Self {
        Self {
            instance_id,
            cluster_id,
            version_id,
            namespace,
            release_name,
            secret_name,
            chart_reference,
            chart_version,
            values,
            secret_data,
            status: UpgradeStatus::UpdatingSecret,
        }
    }
}

impl WorkflowDefinition for UpgradeManagedServiceWorkflow {
    type Error = Box<dyn StdError>;

    async fn next_operations(
        &mut self,
        _ctx: WorkerContext,
    ) -> Result<Vec<Operations>, Self::Error> {
        match self.status {
            UpgradeStatus::UpdatingSecret => {
                self.status = UpgradeStatus::UpgradingHelm;
                Ok(vec![Operations::UpdateK8sSecret(UpdateK8sSecretOp {
                    namespace: self.namespace.clone(),
                    secret_name: self.secret_name.clone(),
                    data: self.secret_data.clone(),
                    previous_data: None,
                })])
            }
            UpgradeStatus::UpgradingHelm => {
                self.status = UpgradeStatus::UpdatingVersion;
                Ok(vec![Operations::HelmUpgrade(HelmUpgradeOp {
                    release_name: self.release_name.clone(),
                    namespace: self.namespace.clone(),
                    chart_reference: self.chart_reference.clone(),
                    chart_version: self.chart_version.clone(),
                    values: self.values.clone(),
                })])
            }
            UpgradeStatus::UpdatingVersion => {
                self.status = UpgradeStatus::MarkingRunning;
                Ok(vec![Operations::UpdateInstanceVersion(
                    UpdateInstanceVersionOp {
                        instance_id: self.instance_id,
                        version_id: self.version_id,
                        previous_version_id: None,
                    },
                )])
            }
            UpgradeStatus::MarkingRunning => {
                self.status = UpgradeStatus::Done;
                Ok(vec![Operations::UpdateInstanceStatus(
                    UpdateInstanceStatusOp {
                        instance_id: self.instance_id,
                        new_status: ManagedServiceInstanceStatus::Running.to_string(),
                        previous_status: None,
                    },
                )])
            }
            UpgradeStatus::Done => Ok(vec![]),
        }
    }

    fn target_cluster_id(&self) -> Option<Uuid> {
        Some(self.cluster_id)
    }

    fn name(&self) -> &str {
        "UpgradeManagedService"
    }
}
