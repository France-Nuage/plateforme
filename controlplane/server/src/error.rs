//! Error handling and type definitions for the gRPC server.
//!
//! This module defines the central error types used throughout the server
//! application, providing a unified error handling interface that abstracts
//! underlying library-specific errors into domain-appropriate error types.

/// Application-level error types for the gRPC server.
///
/// This enumeration represents all possible error conditions that can occur
/// during server operations, providing a centralized error handling mechanism
/// that abstracts underlying library-specific errors into domain-specific
/// error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Transport layer errors from the underlying gRPC transport.
    ///
    /// This variant encapsulates errors that occur during network
    /// communication, server binding, connection handling, and other
    /// transport-related operations managed by the tonic transport layer.
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}
