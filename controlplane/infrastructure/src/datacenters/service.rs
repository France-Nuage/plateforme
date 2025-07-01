use database::Persistable;

use crate::{Datacenter, Problem};

/// Expose functions for interacting with datacenters.
pub struct DatacenterService {
    pool: sqlx::PgPool,
}

impl DatacenterService {
    /// List all datacenters.
    pub async fn list(&self) -> Result<Vec<Datacenter>, Problem> {
        Datacenter::list(&self.pool).await.map_err(Into::into)
    }

    /// Create a datacenter service.
    pub fn new(pool: sqlx::PgPool) -> DatacenterService {
        DatacenterService { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list(pool: sqlx::PgPool) {
        // Arrange the service
        let service = DatacenterService::new(pool.clone());
        let model = Datacenter::factory().create(&pool).await.unwrap();

        // Act the call to the list method
        let result = service.list().await;

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![model]);
    }
}
