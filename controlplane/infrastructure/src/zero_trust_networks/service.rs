use fabrique::Query;

use crate::{Problem, ZeroTrustNetwork};

/// Expose functions for interacting with zero trust networks.
pub struct ZeroTrustNetworkService {
    pool: sqlx::PgPool,
}

impl ZeroTrustNetworkService {
    /// List all zero trust networks.
    pub async fn list(&self) -> Result<Vec<ZeroTrustNetwork>, Problem> {
        ZeroTrustNetwork::query()
            .select()
            .get(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// Create a new zero trust network service.
    pub fn new(pool: sqlx::PgPool) -> ZeroTrustNetworkService {
        ZeroTrustNetworkService { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZeroTrustNetworkType;
    use fabrique::Factory;
    use frn_core::resourcemanager::Organization;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list(pool: sqlx::PgPool) {
        // Arrange the service
        let service = ZeroTrustNetworkService::new(pool.clone());
        let model = ZeroTrustNetwork::factory()
            .for_organization(Organization::factory().parent_id(None))
            .for_zero_trust_network_type(ZeroTrustNetworkType::factory())
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
