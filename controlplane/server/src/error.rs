//! Error handling and type definitions for the gRPC server.
//!
//! This module defines the central error types used throughout the server
//! application, providing a unified error handling interface that abstracts
//! underlying library-specific errors into domain-appropriate error types.

use std::net::AddrParseError;

/// Application-level error types for the gRPC server.
///
/// This enumeration represents all possible error conditions that can occur
/// during server operations, providing a centralized error handling mechanism
/// that abstracts underlying library-specific errors into domain-specific
/// error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("core error: {0}")]
    Core(#[from] frn_core::Error),

    /// Socket address parsing errors.
    ///
    /// This variant occurs when attempting to parse an invalid socket address
    /// string, typically during configuration loading or server setup.
    #[error("address parse error: {0}")]
    InvalidAddress(#[from] AddrParseError),

    /// Input/output errors from system operations.
    ///
    /// This variant encapsulates errors that occur during I/O operations
    /// such as file system access, network operations, or other system-level
    /// operations that return standard I/O errors.
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Database migration errors.
    ///
    /// This variant encapsulates errors that occur during database migration
    /// operations, including schema validation failures, migration script
    /// execution errors, and migration state inconsistencies managed by SQLx.
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Transport layer errors from the underlying gRPC transport.
    ///
    /// This variant encapsulates errors that occur during network
    /// communication, server binding, connection handling, and other
    /// transport-related operations managed by the tonic transport layer.
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}
