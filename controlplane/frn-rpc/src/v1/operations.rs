//! Operations gRPC service implementation.
//!
//! Provides access to long-running operations that synchronize Control Plane
//! state with external systems (SpiceDB, Pangolin, Hoop, Kubernetes).

use std::time::SystemTime;

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::error::Error;

tonic::include_proto!("francenuage.fr.operations.v1");

impl From<frn_core::operations::OperationStatus> for OperationStatus {
    fn from(value: frn_core::operations::OperationStatus) -> Self {
        match value {
            frn_core::operations::OperationStatus::Pending => OperationStatus::Pending,
            frn_core::operations::OperationStatus::Running => OperationStatus::Running,
            frn_core::operations::OperationStatus::Succeeded => OperationStatus::Succeeded,
            frn_core::operations::OperationStatus::Failed => OperationStatus::Failed,
            frn_core::operations::OperationStatus::Cancelled => OperationStatus::Cancelled,
        }
    }
}

impl From<frn_core::operations::OperationType> for OperationType {
    fn from(value: frn_core::operations::OperationType) -> Self {
        match value {
            frn_core::operations::OperationType::SpiceDbWriteRelationship => {
                OperationType::SpiceDbWriteRelationship
            }
            frn_core::operations::OperationType::SpiceDbDeleteRelationship => {
                OperationType::SpiceDbDeleteRelationship
            }
            frn_core::operations::OperationType::PangolinInviteUser => {
                OperationType::PangolinInviteUser
            }
            frn_core::operations::OperationType::PangolinRemoveUser => {
                OperationType::PangolinRemoveUser
            }
            frn_core::operations::OperationType::PangolinUpdateUser => {
                OperationType::PangolinUpdateUser
            }
            frn_core::operations::OperationType::HoopCreateAgent => OperationType::HoopCreateAgent,
            frn_core::operations::OperationType::HoopDeleteAgent => OperationType::HoopDeleteAgent,
            frn_core::operations::OperationType::HoopCreateConnection => {
                OperationType::HoopCreateConnection
            }
            frn_core::operations::OperationType::HoopDeleteConnection => {
                OperationType::HoopDeleteConnection
            }
            frn_core::operations::OperationType::K8sCreateNamespaceAccess => {
                OperationType::K8sCreateNamespaceAccess
            }
            frn_core::operations::OperationType::K8sDeleteNamespaceAccess => {
                OperationType::K8sDeleteNamespaceAccess
            }
        }
    }
}

impl From<frn_core::operations::TargetBackend> for TargetBackend {
    fn from(value: frn_core::operations::TargetBackend) -> Self {
        match value {
            frn_core::operations::TargetBackend::SpiceDb => TargetBackend::SpiceDb,
            frn_core::operations::TargetBackend::Pangolin => TargetBackend::Pangolin,
            frn_core::operations::TargetBackend::Hoop => TargetBackend::Hoop,
            frn_core::operations::TargetBackend::Kubernetes => TargetBackend::Kubernetes,
        }
    }
}

fn json_to_struct(value: serde_json::Value) -> prost_types::Struct {
    let map = value.as_object().map(|obj| {
        obj.iter()
            .map(|(k, v)| (k.clone(), json_to_value(v.clone())))
            .collect()
    });

    prost_types::Struct {
        fields: map.unwrap_or_default(),
    }
}

fn json_to_value(value: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;

    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.into_iter().map(json_to_value).collect(),
        }),
        serde_json::Value::Object(_) => Kind::StructValue(json_to_struct(value)),
    };

    prost_types::Value { kind: Some(kind) }
}

impl From<frn_core::operations::Operation> for Operation {
    fn from(value: frn_core::operations::Operation) -> Self {
        Operation {
            name: value.name,
            operation_type: OperationType::from(value.operation_type) as i32,
            target_backend: TargetBackend::from(value.target_backend) as i32,
            resource_type: value.resource_type,
            resource_id: value.resource_id.to_string(),
            status: OperationStatus::from(value.status) as i32,
            input: Some(json_to_struct(value.input)),
            output: value.output.map(json_to_struct),
            error: value.error_code.map(|code| OperationError {
                code,
                message: value.error_message.unwrap_or_default(),
            }),
            attempt_count: value.attempt_count,
            max_attempts: value.max_attempts,
            created_at: Some(SystemTime::from(value.created_at).into()),
            started_at: value.started_at.map(|t| SystemTime::from(t).into()),
            completed_at: value.completed_at.map(|t| SystemTime::from(t).into()),
            next_retry_at: value.next_retry_at.map(|t| SystemTime::from(t).into()),
        }
    }
}

/// Operations service implementation.
pub struct Operations {
    db: Pool<Postgres>,
}

impl Operations {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl operations_server::Operations for Operations {
    async fn get(
        &self,
        request: Request<GetOperationRequest>,
    ) -> Result<Response<Operation>, Status> {
        let GetOperationRequest { name } = request.into_inner();

        let operation = frn_core::operations::Operation::find_by_name(&self.db, &name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Error::NotFound(name))?;

        Ok(Response::new(operation.into()))
    }

    async fn list(
        &self,
        request: Request<ListOperationsRequest>,
    ) -> Result<Response<ListOperationsResponse>, Status> {
        let ListOperationsRequest {
            resource_type,
            resource_id,
            status: _status,
            target_backend: _target_backend,
            page_size: _page_size,
            page_token: _page_token,
        } = request.into_inner();

        // If resource_type and resource_id are provided, filter by resource
        let operations = if let (Some(resource_type), Some(resource_id)) =
            (resource_type, resource_id)
        {
            let resource_id = Uuid::parse_str(&resource_id)
                .map_err(|_| Error::MalformedId(resource_id.clone()))?;

            frn_core::operations::Operation::list_by_resource(&self.db, &resource_type, resource_id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
        } else {
            // For now, return empty if no filter is provided
            // In a full implementation, we'd support pagination and other filters
            vec![]
        };

        Ok(Response::new(ListOperationsResponse {
            operations: operations.into_iter().map(Into::into).collect(),
            next_page_token: String::new(),
        }))
    }

    async fn cancel(
        &self,
        request: Request<CancelOperationRequest>,
    ) -> Result<Response<Operation>, Status> {
        let CancelOperationRequest { name } = request.into_inner();

        let operation = frn_core::operations::Operation::find_by_name(&self.db, &name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Error::NotFound(name.clone()))?;

        let cancelled = operation
            .cancel(&self.db)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !cancelled {
            return Err(Status::failed_precondition(
                "operation cannot be cancelled (already in terminal state)",
            ));
        }

        // Fetch the updated operation
        let updated = frn_core::operations::Operation::find_by_name(&self.db, &name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Error::NotFound(name))?;

        Ok(Response::new(updated.into()))
    }
}
