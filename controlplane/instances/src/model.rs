//! Database model for instance entities.

use uuid::Uuid;

#[derive(Debug, Default, sqlx::FromRow)]
pub struct Instance {
    /// Unique identifier for the instance
    pub id: Uuid,
    /// Reference to the hypervisor hosting this instance
    pub hypervisor_id: Uuid,
    /// ID used by the hypervisor to identify this instance remotely
    pub distant_id: String,
}
