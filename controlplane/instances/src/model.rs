//! Database model for instance entities.

use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Instance {
    pub id: Uuid,
    pub hypervisor_id: Uuid,
    pub distant_id: String,
}
