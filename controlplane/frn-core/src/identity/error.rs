use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    /// JWT header is missing the required "kid" (Key ID) claim.
    #[error("missing kid in jwt header")]
    MissingKid,

    /// Failed to parse the JWK Set from the provider's JWKS endpoint.
    #[error("unparsable jwks for url {0}")]
    UnparsableJwks(String),

    /// Failed to parse OIDC provider metadata from the discovery endpoint.
    #[error("unparsable metadata for oidc provider {0}")]
    UnparsableOidcMetadata(String),

    /// Cannot establish a network connection to the OIDC provider.
    #[error("unreachable oidc provider {0}")]
    UnreachableOidcProvider(String),
}
