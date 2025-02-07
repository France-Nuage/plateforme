use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub node_id: String,
    pub pve_vm_id: String,
    pub status: InstanceStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InstanceStatus {
    Provisioning,
    Staging,
    Running,
    Stopping,
    Stopped,
    Terminated,
    Deleting,
    Deleted,
}
