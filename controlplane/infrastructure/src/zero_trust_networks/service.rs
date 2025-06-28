use database::Persistable;

use crate::{Problem, ZeroTrustNetwork};

/// Expose functions for interacting with zero trust networks.
pub struct ZeroTrustNetworkService {
    pool: sqlx::PgPool,
}

impl ZeroTrustNetworkService {
    /// List all zero trust networks.
    pub async fn list(&self) -> Result<Vec<ZeroTrustNetwork>, Problem> {
        ZeroTrustNetwork::list(&self.pool).await.map_err(Into::into)
    }

    /// Create a new zero trust network service.
    pub fn new(pool: sqlx::PgPool) -> ZeroTrustNetworkService {
        ZeroTrustNetworkService { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list(pool: sqlx::PgPool) {
        // Arrange the service
        let service = ZeroTrustNetworkService::new(pool.clone());
        let model = ZeroTrustNetwork::factory()
            .for_default_organization()
            .for_default_zero_trust_network_type()
            .create(&pool)
            .await
            .unwrap();

        // Act the call to the list method
        let result = service.list().await;

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![model]);
    }
}
