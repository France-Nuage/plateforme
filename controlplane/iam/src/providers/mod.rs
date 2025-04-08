//! OAuth2 provider implementations.
//!
//! This module contains implementations of the AuthProvider trait
//! for various OAuth2 providers.

pub mod google;

// Re-export providers for convenience
pub use google::GoogleAuthProvider;