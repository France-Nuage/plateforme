use frn_core::authorization::Authorize;
use frn_core::identity::IAM;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::error::Error;

tonic::include_proto!("francenuage.fr.longrunning.v1");

/// gRPC service for long-running operations.
pub struct Operations<Auth: Authorize> {
    iam: IAM,
    operations: frn_core::longrunning::Operations<Auth>,
}

impl<Auth: Authorize> Operations<Auth> {
    pub fn new(iam: IAM, operations: frn_core::longrunning::Operations<Auth>) -> Self {
        Self { iam, operations }
    }
}

impl From<frn_core::longrunning::Operation> for Operation {
    fn from(op: frn_core::longrunning::Operation) -> Self {
        Operation {
            name: op.id.to_string(),
            done: Some(op.completed_at.is_some()),
        }
    }
}

#[tonic::async_trait]
impl<Auth: Authorize + 'static> operations_server::Operations for Operations<Auth> {
    async fn get(
        &self,
        request: Request<GetOperationRequest>,
    ) -> Result<Response<Operation>, Status> {
        let principal = self.iam.principal(&request).await?;
        let name = request.into_inner().name;

        let id = Uuid::parse_str(&name).map_err(|_| Error::MalformedId(name))?;

        let operation = self.operations.clone().get(&principal, id).await?;

        Ok(Response::new(operation.into()))
    }

    async fn wait(
        &self,
        request: Request<WaitOperationRequest>,
    ) -> Result<Response<Operation>, Status> {
        let principal = self.iam.principal(&request).await?;
        let WaitOperationRequest { name, timeout } = request.into_inner();

        let id = Uuid::parse_str(&name).map_err(|_| Error::MalformedId(name.clone()))?;
        let timeout = timeout.and_then(|d| d.try_into().ok());

        let operation = self
            .operations
            .clone()
            .wait(&principal, id, timeout)
            .await?;

        Ok(Response::new(operation.into()))
    }
}
