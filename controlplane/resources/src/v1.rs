//! Generated code and type conversions for the resources gRPC API.
//!
//! This module includes the generated code from the resources.proto file
//! and provides type conversions between API types and model types.

use std::time::SystemTime;

// Include the generated code from the hypervisors.proto file
tonic::include_proto!("francenuage.fr.api.controlplane.v1.resources");

/// Convert between model and protobuf types
impl From<frn_core::models::Organization> for Organization {
    fn from(org: frn_core::models::Organization) -> Self {
        Self {
            id: org.id.to_string(),
            name: org.name,
            created_at: Some(SystemTime::from(org.created_at).into()),
            updated_at: Some(SystemTime::from(org.updated_at).into()),
        }
    }
}

/// Convert between model and protobuf types
impl From<frn_core::models::Project> for Project {
    fn from(project: frn_core::models::Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            organization_id: project.organization_id.to_string(),
            created_at: Some(SystemTime::from(project.created_at).into()),
            updated_at: Some(SystemTime::from(project.updated_at).into()),
        }
    }
}
