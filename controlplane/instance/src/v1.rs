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
impl From<hypervisor_connector::InstanceInfo> for InstanceInfo {
    fn from(value: hypervisor_connector::InstanceInfo) -> Self {
        InstanceInfo {
            id: value.id,
            status: value.status as i32,
            max_cpu_cores: value.max_cpu_cores,
            cpu_usage_percent: value.cpu_usage_percent,
            max_memory_bytes: value.max_memory_bytes,
            memory_usage_bytes: value.memory_usage_bytes,
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

/// Converts a `Result<Vec<hypervisor_connector::InstanceInfo>, hypervisor_connector::Problem>`
/// into a `v1::ListInstancesResponse`.
impl From<Result<Vec<hypervisor_connector::InstanceInfo>, hypervisor_connector::Problem>>
    for ListInstancesResponse
{
    fn from(
        value: Result<Vec<hypervisor_connector::InstanceInfo>, hypervisor_connector::Problem>,
    ) -> Self {
        ListInstancesResponse {
            result: match value {
                Ok(instances) => Some(crate::v1::list_instances_response::Result::Success(
                    InstanceList {
                        instances: instances.into_iter().map(Into::into).collect(),
                    },
                )),
                Err(_) => todo!(),
            },
        }
    }
}

/// Converts a `Result<(), hypervisor_connector::Problem>` into a `v1::StartInstanceResponse`.
impl From<Result<(), hypervisor_connector::Problem>> for StartInstanceResponse {
    fn from(value: Result<(), hypervisor_connector::Problem>) -> Self {
        StartInstanceResponse {
            result: match value {
                Ok(()) => Some(crate::v1::start_instance_response::Result::Success(())),
                Err(_) => todo!(),
            },
        }
    }
}

/// Converts a `Result<(), hypervisor_connector::Problem>` into a `v1::StopInstanceResponse`.
impl From<Result<(), hypervisor_connector::Problem>> for StopInstanceResponse {
    fn from(value: Result<(), hypervisor_connector::Problem>) -> Self {
        StopInstanceResponse {
            result: match value {
                Ok(()) => Some(crate::v1::stop_instance_response::Result::Success(())),
                Err(_) => todo!(),
            },
        }
    }
}
