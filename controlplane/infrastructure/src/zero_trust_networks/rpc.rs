use super::ZeroTrustNetworkService;
use crate::v1::{
    ListZeroTrustNetworksRequest, ListZeroTrustNetworksResponse,
    zero_trust_networks_server::ZeroTrustNetworks,
};
use tonic::{Request, Response, Status};

pub struct ZeroTrustNetworkRpcService {
    service: ZeroTrustNetworkService,
}

impl ZeroTrustNetworkRpcService {
    /// Create a new instance of the ZeroTrustNetworks gRPC service.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            service: ZeroTrustNetworkService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl ZeroTrustNetworks for ZeroTrustNetworkRpcService {
    async fn list(
        &self,
        _: Request<ListZeroTrustNetworksRequest>,
    ) -> Result<Response<ListZeroTrustNetworksResponse>, Status> {
        let models = self.service.list().await?;

        Ok(Response::new(ListZeroTrustNetworksResponse {
            zero_trust_networks: models.into_iter().map(Into::into).collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZeroTrustNetwork, ZeroTrustNetworkType};
    use fabrique::Factory;
    use frn_core::resourcemanager::Organization;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_organizations_works(pool: sqlx::PgPool) {
        // Arrange the test
        let model = ZeroTrustNetwork::factory()
            .for_organization(Organization::factory().parent_id(None))
            .for_zero_trust_network_type(ZeroTrustNetworkType::factory())
            .create(&pool)
            .await
            .unwrap();
        let service = ZeroTrustNetworkRpcService::new(pool);

        // Act the call to the list procedure
        let result = service
            .list(Request::new(ListZeroTrustNetworksRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().into_inner(),
            ListZeroTrustNetworksResponse {
                zero_trust_networks: vec![model.into()],
            }
        )
    }
}
