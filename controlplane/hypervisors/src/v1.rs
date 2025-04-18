//! Generated code and type conversions for the hypervisors gRPC API.
//!
//! This module includes the generated code from the hypervisors.proto file
//! and provides type conversions between API types and model types.

use sea_orm::ActiveValue;
use uuid::Uuid;

// Include the generated code from the hypervisors.proto file
tonic::include_proto!("francenuage.fr.api.controlplane.v1.hypervisors");

/// Converts a RegisterHypervisorRequest to an ActiveModel for database persistence.
///
/// This implementation maps the fields from the API request to the corresponding
/// fields in the database model, generating a new UUID for the hypervisor ID.
impl From<RegisterHypervisorRequest> for crate::model::ActiveModel {
    fn from(value: RegisterHypervisorRequest) -> Self {
        crate::model::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            url: ActiveValue::Set(value.url),
            authentication_token: ActiveValue::Set(value.authentication_token),
            storage_name: ActiveValue::Set(value.storage_name),
        }
    }
}

/// Converts a database Model to a Hypervisor API type.
///
/// This implementation maps the fields from the database model to the corresponding
/// fields in the API response type, which currently only includes the URL.
impl From<crate::model::Model> for Hypervisor {
    fn from(value: crate::model::Model) -> Self {
        Hypervisor { url: value.url }
    }
}
