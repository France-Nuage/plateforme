use crate::ZeroTrustNetworkTypeService;
use crate::v1::{
    ListZeroTrustNetworkTypesRequest, ListZeroTrustNetworkTypesResponse,
    zero_trust_network_types_server::ZeroTrustNetworkTypes,
};
use tonic::{Request, Response, Status};

pub struct ZeroTrustNetworkTypeRpcService {
    service: ZeroTrustNetworkTypeService,
}

impl ZeroTrustNetworkTypeRpcService {
    /// Create a new instance of the ZeroTrustNetworkTypes gRPC service.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            service: ZeroTrustNetworkTypeService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl ZeroTrustNetworkTypes for ZeroTrustNetworkTypeRpcService {
    async fn list(
        &self,
        _: Request<ListZeroTrustNetworkTypesRequest>,
    ) -> Result<Response<ListZeroTrustNetworkTypesResponse>, Status> {
        let zero_trust_network_types = self.service.list().await?;

        Ok(Response::new(ListZeroTrustNetworkTypesResponse {
            zero_trust_network_types: zero_trust_network_types
                .into_iter()
                .map(Into::into)
                .collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZeroTrustNetworkType;
    use fabrique::Factory;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_list_organizations_works(pool: sqlx::PgPool) {
        // Arrange the test
        let model = ZeroTrustNetworkType::factory().create(&pool).await.unwrap();
        let service = ZeroTrustNetworkTypeRpcService::new(pool);

        // Act the call to the list procedure
        let result = service
            .list(Request::new(ListZeroTrustNetworkTypesRequest::default()))
            .await;

        // Assert the procedure result
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().into_inner(),
            ListZeroTrustNetworkTypesResponse {
                zero_trust_network_types: vec![model.into()],
            }
        )
    }
}
