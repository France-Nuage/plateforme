//! OpenID Connect Discovery protocol implementation.
//!
//! This module provides data structures for parsing OpenID Connect Provider Metadata
//! according to the [OpenID Connect Discovery 1.0] specification. It enables automatic
//! discovery of OIDC provider configuration from well-known endpoints.
//!
//! ## Overview
//!
//! OpenID Connect Discovery allows clients to dynamically discover information about
//! an OpenID Provider, such as:
//! - Issuer identifier
//! - Authorization and token endpoint URLs  
//! - JWK Set location for token verification
//! - Supported scopes, response types, and algorithms
//!
//! ## Well-known Endpoint
//!
//! OIDC providers publish their metadata at a standardized location:
//! ```text
//! https://example.com/.well-known/openid_configuration
//! ```
//!
//! [OpenID Connect Discovery 1.0]: https://openid.net/specs/openid-connect-discovery-1_0.html

use serde::Deserialize;

/// OpenID Connect Provider Metadata structure.
///
/// Represents the metadata document returned by an OpenID Connect provider's
/// discovery endpoint. This structure contains essential configuration information
/// needed to interact with the provider, particularly for JWT token validation.
///
/// ## Specification Compliance
///
/// This struct implements the Provider Metadata format defined in the
/// [OpenID Connect Discovery 1.0 specification](https://openid.net/specs/openid-connect-discovery-1_0.html#ProviderMetadata).
/// While the full specification includes many optional fields, this implementation focuses on
/// the core fields required for JWT validation workflows.
///
/// ## Required Fields
///
/// According to the specification, the following fields are **REQUIRED**:
/// - [`issuer`] - The provider's issuer identifier
/// - [`jwks_uri`] - Location of the provider's JWK Set
///
/// Additional optional fields can be added to this struct as needed without
/// breaking compatibility, since serde will ignore unknown fields during
/// deserialization.
///
/// ## Security Considerations
///
/// - Always verify that the [`issuer`] field matches the expected provider
/// - Ensure [`jwks_uri`] uses HTTPS to prevent man-in-the-middle attacks
/// - Cache metadata appropriately but respect provider's cache directives
///
/// [`issuer`]: OpenIDProviderMetadata::issuer
/// [`jwks_uri`]: OpenIDProviderMetadata::jwks_uri
#[derive(Debug, Deserialize)]
pub struct OpenIDProviderMetadata {
    /// REQUIRED. URL using the https scheme with no query or fragment
    /// components that the OP asserts as its Issuer Identifier. If Issuer
    /// discovery is supported (see Section 2), this value MUST be identical
    /// to the issuer value returned by WebFinger. This also MUST be identical
    /// to the iss Claim value in ID Tokens issued from this Issuer.
    pub issuer: String,

    /// REQUIRED. URL of the OP's JWK Set [JWK] document, which MUST use the
    /// https scheme. This contains the signing key(s) the RP uses to validate
    /// signatures from the OP. The JWK Set MAY also contain the Server's
    /// encryption key(s), which are used by RPs to encrypt requests to the
    /// Server. When both signing and encryption keys are made available, a use
    /// (public key use) parameter value is REQUIRED for all keys in the
    /// referenced JWK Set to indicate each key's intended usage. Although some
    /// algorithms allow the same key to be used for both signatures and
    /// encryption, doing so is NOT RECOMMENDED, as it is less secure. The JWK
    /// x5c parameter MAY be used to provide X.509 representations of keys
    /// provided. When used, the bare key values MUST still be present and MUST
    /// match those in the certificate. The JWK Set MUST NOT contain private or
    /// symmetric key values.
    pub jwks_uri: String,
}
