use frn_core::resourcemanager::Organization;
use sqlx::PgPool;

use crate::Problem;

pub struct OrganizationService {
    pool: PgPool,
}

impl OrganizationService {
    /// List all organizations.
    pub async fn list(&self) -> Result<Vec<Organization>, Problem> {
        crate::organizations::repository::list(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// Create a new organization.
    pub async fn create(&self, organization: Organization) -> Result<Organization, Problem> {
        crate::organizations::repository::create(&self.pool, organization)
            .await
            .map_err(Into::into)
    }

    /// Create a new organization service.
    pub fn new(pool: PgPool) -> OrganizationService {
        OrganizationService { pool }
    }
}
