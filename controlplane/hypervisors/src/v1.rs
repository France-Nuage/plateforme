//! Generated code and type conversions for the hypervisors gRPC API.
//!
//! This module includes the generated code from the hypervisors.proto file
//! and provides type conversions between API types and model types.

use tonic::Status;
use uuid::Uuid;

// Include the generated code from the hypervisors.proto file
tonic::include_proto!("francenuage.fr.api.controlplane.v1.hypervisors");

/// Converts a RegisterHypervisorRequest to an ActiveModel for database persistence.
///
/// This implementation maps the fields from the API request to the corresponding
/// fields in the database model, generating a new UUID for the hypervisor ID.
impl TryFrom<RegisterHypervisorRequest> for crate::model::Hypervisor {
    type Error = Status;

    fn try_from(value: RegisterHypervisorRequest) -> Result<Self, Self::Error> {
        let organization_id = Uuid::parse_str(&value.organization_id)
            .map_err(|_| Status::invalid_argument("Invalid organization_id format"))?;

        Ok(crate::model::Hypervisor {
            id: Uuid::new_v4(),
            organization_id,
            url: value.url,
            authorization_token: value.authorization_token,
            storage_name: value.storage_name,
        })
    }
}

/// Converts a database Model to a Hypervisor API type.
///
/// This implementation maps the fields from the database model to the corresponding
/// fields in the API response type, which currently only includes the URL.
impl From<crate::model::Hypervisor> for Hypervisor {
    fn from(value: crate::model::Hypervisor) -> Self {
        Hypervisor {
            id: value.id.to_string(),
            organization_id: value.organization_id.to_string(),
            storage_name: value.storage_name,
            url: value.url,
        }
    }
}
