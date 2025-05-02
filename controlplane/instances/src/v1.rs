//! # FranceNuage Control Plane API services v1
//!
//! This module provides the protocol buffer definitions and type conversions for the v1
//! version of the FranceNuage control plane services API, specifically for instance management.
//!
//! ## Overview
//!
//! The module includes:
//! - Protocol buffer generated code for the instance management API
//! - Conversion implementations between `hypervisor_connector` types and protocol message types
//!
//! ## Type Conversions
//!
//! This module implements the `From` trait for various types to enable seamless conversion
//! between internal application types and the protocol-compatible message types:
//!
//! - `hypervisor_connector::InstanceInfo` → `v1::InstanceInfo`
//! - `hypervisor_connector::InstanceStatus` → `v1::InstanceStatus`
//! - `Result<Vec<hypervisor_connector::InstanceInfo>, hypervisor_connector::Problem>` → `v1::ListInstancesResponse`
//! - `Result<(), hypervisor_connector::Problem>` → `v1::StartInstanceResponse`
//! - `Result<(), hypervisor_connector::Problem>` → `v1::StopInstanceResponse`
//!
//! ## Usage Notes
//!
//! The conversions in this module simplify implementing gRPC service handlers by providing
//! automatic conversion from internal result types to protocol message responses.

tonic::include_proto!("francenuage.fr.api.controlplane.v1.instances");

/// Converts a InstanceInfo struct into a protocol compatible `v1::InstanceInfo`.
impl From<hypervisor_connector::InstanceInfo> for Instance {
    fn from(value: hypervisor_connector::InstanceInfo) -> Self {
        Instance {
            id: value.id,
            status: value.status as i32,
            max_cpu_cores: value.max_cpu_cores,
            cpu_usage_percent: value.cpu_usage_percent,
            max_memory_bytes: value.max_memory_bytes,
            memory_usage_bytes: value.memory_usage_bytes,
            name: value.name,
        }
    }
}

/// Converts a `crate::model::Instance` into a protocol compatible `v1::InstanceInfo`.
impl From<crate::model::Instance> for Instance {
    fn from(value: crate::model::Instance) -> Self {
        Instance {
            id: value.id.to_string(),
            ..Default::default() // TODO: return concrete values
        }
    }
}

/// Converts a `hypervisor_connector::InstanceStatus` into a protocol compatible
/// `v1::InstanceStatus`.
impl From<hypervisor_connector::InstanceStatus> for InstanceStatus {
    fn from(value: hypervisor_connector::InstanceStatus) -> Self {
        match value {
            hypervisor_connector::InstanceStatus::Running => InstanceStatus::Running,
            hypervisor_connector::InstanceStatus::Stopped => InstanceStatus::Stopped,
        }
    }
}

/// Converts a `v1::CreateInstanceRequest` into a `hypervisor_connector::InstanceConfig`.
impl From<CreateInstanceRequest> for hypervisor_connector::InstanceConfig {
    fn from(value: CreateInstanceRequest) -> Self {
        hypervisor_connector::InstanceConfig {
            id: String::from("invalid"),
            cores: value.cpu_cores as u8,
            disk_image: value.image,
            memory: value.memory_bytes as u32,
            name: value.name,
            snippet: value.snippet,
        }
    }
}
