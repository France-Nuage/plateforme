use fabrique::Query;

use crate::{Problem, ZeroTrustNetworkType};

/// Expose functions for interacting with zero trust network types.
pub struct ZeroTrustNetworkTypeService {
    pool: sqlx::PgPool,
}

impl ZeroTrustNetworkTypeService {
    /// List all zero trust network types.
    pub async fn list(&self) -> Result<Vec<ZeroTrustNetworkType>, Problem> {
        ZeroTrustNetworkType::query()
            .select()
            .get(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// Create a new zero trust network type service.
    pub fn new(pool: sqlx::PgPool) -> ZeroTrustNetworkTypeService {
        ZeroTrustNetworkTypeService { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabrique::Factory;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list(pool: sqlx::PgPool) {
        // Arrange the service
        let service = ZeroTrustNetworkTypeService::new(pool.clone());
        let model = ZeroTrustNetworkType::factory().create(&pool).await.unwrap();

        // Act the call to the list method
        let result = service.list().await;

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![model]);
    }
}
