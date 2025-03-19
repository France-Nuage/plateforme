use tonic::{Request, Response, Status};

use super::proto::instance_server::Instance;
use super::proto::{InstanceStatusRequest, InstanceStatusResponse};

#[derive(Debug, Default)]
pub struct InstanceService {}

#[tonic::async_trait]
impl Instance for InstanceService {
    async fn status(
        &self,
        request: Request<InstanceStatusRequest>,
    ) -> Result<Response<InstanceStatusResponse>, Status> {
        println!("Request: {:?}", request);
        Ok(Response::new(InstanceStatusResponse {
            status: String::from("OK"),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_works() {
        // Arrange a service and a request for the status procedure
        let service = InstanceService::default();
        let request = Request::new(InstanceStatusRequest {
            id: String::from("666"),
        });

        // Act the call to the status procedure
        let result = service.status(request).await;

        // Assert the procedure result
        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_inner().status, String::from("OK"));
    }
}
