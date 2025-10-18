mod error;
pub mod request;
pub mod v1;

/// File descriptor set for gRPC reflection v1.
///
/// This constant contains the encoded file descriptor set for all proto services
/// defined in this crate. It's used by the gRPC reflection v1 service to provide
/// runtime service discovery and schema inspection capabilities.
///
/// The descriptor is generated at build time by the build.rs script and includes
/// the Compute and ResourceManager service definitions.
pub const REFLECTION_DESCRIPTOR_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/reflection_descriptor_v1.bin"));

/// File descriptor set for gRPC reflection v1alpha.
///
/// This constant contains the encoded file descriptor set for all proto services
/// defined in this crate. It's used by the gRPC reflection v1alpha service to provide
/// runtime service discovery and schema inspection capabilities for clients that
/// only support the v1alpha reflection API (like Bruno).
///
/// The descriptor is generated at build time by the build.rs script and includes
/// the Compute and ResourceManager service definitions.
pub const REFLECTION_DESCRIPTOR_V1ALPHA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/reflection_descriptor_v1alpha.bin"
));
