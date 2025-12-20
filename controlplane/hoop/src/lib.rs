//! Hoop.dev API client for SSH bastion integration.
//!
//! This crate provides a client for interacting with the Hoop.dev API to manage
//! SSH bastion agents and connections for VM instances.

pub mod api;

#[cfg(feature = "mock")]
pub mod mock;
