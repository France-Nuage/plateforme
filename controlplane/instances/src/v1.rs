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

use std::time::SystemTime;

tonic::include_proto!("francenuage.fr.api.controlplane.v1.instances");

/// Converts a `crate::model::Instance` into a protocol compatible `v1::InstanceInfo`.
impl From<crate::model::Instance> for Instance {
    fn from(value: crate::model::Instance) -> Self {
        Instance {
            id: value.id.to_string(),
            cpu_usage_percent: value.cpu_usage_percent as f32,
            disk_usage_bytes: 0_u64,
            max_cpu_cores: value.max_cpu_cores as u32,
            max_disk_bytes: 0_u64,
            max_memory_bytes: value.max_memory_bytes as u64,
            memory_usage_bytes: value.memory_usage_bytes as u64,
            ip_v4: String::from(""),
            name: value.name,
            project_id: value.project_id.to_string(),
            status: value.status as i32,
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
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
