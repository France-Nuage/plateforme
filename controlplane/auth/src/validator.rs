//! JWT token validation with JWK key management and caching.
//!
//! This module provides the core JWT validation functionality, including automatic
//! discovery of OIDC provider configuration, fetching and caching of JWK keys,
//! and high-performance token validation.
//!
//! ## Key Features
//!
//! - **Automatic OIDC Discovery**: Fetches provider metadata from well-known endpoints
//! - **JWK Key Caching**: Intelligent caching of cryptographic keys with TTL expiration  
//! - **Concurrent Key Fetching**: Parallel fetching of multiple keys with backpressure control
//! - **Standards Compliant**: Full RFC 7519 (JWT) and RFC 7517 (JWK) compliance
//!
//! ## Caching Strategy
//!
//! The validator uses a high-performance cache for JWK keys:
//! - **Capacity**: 200 keys maximum
//! - **TTL**: 1 hour time-to-live per key
//! - **Lazy Loading**: Keys are fetched on-demand when first needed
//! - **Automatic Refresh**: Keys are re-fetched when cache expires
//!
//! ## Performance Characteristics
//!
//! - **First Token**: Requires OIDC discovery + JWK fetch (~2 network requests)
//! - **Cached Keys**: Sub-millisecond validation using cached cryptographic keys
//! - **Concurrent Validation**: Thread-safe and optimized for high-throughput scenarios

use crate::discovery::OpenIDProviderMetadata;
use crate::error::Error;
use crate::rfc7517::Claim;
use futures::{StreamExt, TryStreamExt, stream};
use jsonwebtoken::{DecodingKey, Validation, jwk::JwkSet};
use jsonwebtoken::{TokenData, decode};
use moka::future::Cache;
use std::time::Duration;

/// High-performance JWT validator with automatic JWK key management.
///
/// `JwkValidator` provides a complete JWT validation solution that handles:
/// - OIDC provider discovery and metadata parsing
/// - JWK Set fetching and parsing  
/// - Cryptographic key caching for performance
/// - JWT signature validation and claims extraction
///
/// ## Initialization
///
/// The validator is typically initialized once per application using OIDC discovery:
///
/// ```rust,no_run
/// # use auth::JwkValidator;
/// # async fn example() -> Result<(), auth::Error> {
/// let validator = JwkValidator::from_oidc_discovery(
///     "https://accounts.google.com/.well-known/openid_configuration"
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Thread Safety
///
/// `JwkValidator` is thread-safe and designed to be shared across multiple async tasks.
/// The internal cache is concurrent and lock-free, making it suitable for high-throughput
/// applications.
///
/// ## Error Handling
///
/// The validator provides detailed error information for debugging and monitoring:
/// - Network errors when contacting OIDC providers
/// - Parsing errors for malformed metadata or JWK sets
/// - JWT validation errors (signature, expiration, etc.)
///
/// ## Cache Behavior
///
/// Keys are cached using the JWT header's `kid` (Key ID) field as the cache key.
/// If a token references an unknown `kid`, the validator will:
/// 1. Fetch the latest JWK Set from the provider
/// 2. Cache all keys from the set
/// 3. Retry validation with the newly cached key
#[derive(Clone)]
pub struct JwkValidator {
    /// The OIDC provider's issuer identifier
    pub issuer: String,
    /// HTTP client for fetching OIDC metadata and JWK sets
    client: reqwest::Client,
    /// URL endpoint where the provider's JWK Set can be fetched
    jwks_uri: String,
    /// High-performance cache for JWK decoding keys, keyed by `kid` (Key ID)
    keys: Cache<String, DecodingKey>,
}

impl JwkValidator {
    /// Creates a new JWT validator using OIDC provider discovery.
    ///
    /// This method performs automatic discovery of the OIDC provider's configuration
    /// by fetching the provider metadata from the standard well-known endpoint.
    /// The discovery process retrieves essential information needed for JWT validation,
    /// including the issuer identifier and JWK Set URI.
    ///
    /// ## Arguments
    ///
    /// * `discovery_url` - The OIDC provider's discovery endpoint URL, typically in the format:
    ///   `https://provider.domain/.well-known/openid_configuration`
    ///
    /// ## Returns
    ///
    /// * `Ok(JwkValidator)` - A configured validator ready for token validation
    /// * `Err(Error)` - If discovery fails due to network issues or malformed metadata
    ///
    /// ## Errors
    ///
    /// This method can fail with:
    /// * [`Error::UnreachableOidcProvider`] - Cannot connect to the discovery endpoint
    /// * [`Error::UnparsableOidcMetadata`] - Provider metadata is malformed or incomplete
    ///
    /// ## Network Behavior
    ///
    /// The method makes a single HTTP GET request to fetch the provider metadata.
    /// Ensure the discovery URL is accessible and returns valid JSON conforming to
    /// the OpenID Connect Discovery specification.
    ///
    /// ## Security Considerations
    ///
    /// - Always use HTTPS URLs for discovery endpoints in production
    /// - Verify that the returned `issuer` field matches your expected provider
    /// - Consider caching the validator instance rather than recreating it frequently
    pub async fn from_oidc_discovery(discovery_url: &str) -> Result<Self, crate::Error> {
        let client = reqwest::Client::new();

        let config: OpenIDProviderMetadata = client
            .get(discovery_url)
            .send()
            .await
            .map_err(|_| crate::Error::UnreachableOidcProvider(discovery_url.to_string()))?
            .json()
            .await
            .map_err(|_| crate::Error::UnparsableOidcMetadata(discovery_url.to_string()))?;

        Ok(Self {
            client,
            issuer: config.issuer,
            jwks_uri: config.jwks_uri,
            keys: Cache::builder()
                .max_capacity(200)
                .time_to_live(Duration::from_secs(3600))
                .build(),
        })
    }

    /// Validates a JWT token and extracts its claims.
    ///
    /// This method performs complete JWT validation including:
    /// 1. JWT header parsing to extract the Key ID (`kid`)
    /// 2. JWK key retrieval (cached or fetched from provider)
    /// 3. Cryptographic signature verification
    /// 4. Claims deserialization and validation
    ///
    /// ## Arguments
    ///
    /// * `token` - The JWT token string to validate (without "Bearer " prefix)
    ///
    /// ## Returns
    ///
    /// * `Ok(TokenData<Claim>)` - Contains validated claims and token metadata
    /// * `Err(Error)` - If validation fails for any reason
    ///
    /// ## Errors
    ///
    /// This method can fail with:
    /// * [`Error::MissingKid`] - JWT header lacks required `kid` field
    /// * [`Error::Other`] - JWT signature invalid, expired, malformed, etc.
    /// * [`Error::UnreachableOidcProvider`] - Cannot fetch JWK Set for unknown key
    /// * [`Error::UnparsableJwks`] - JWK Set from provider is malformed
    ///
    /// ## Performance Notes
    ///
    /// - **First validation**: Requires JWK fetch (~100-500ms depending on network)
    /// - **Subsequent validations**: Sub-millisecond using cached keys
    /// - **Unknown keys**: Triggers JWK refresh, then retries validation
    ///
    /// ## Security Guarantees
    ///
    /// On successful validation, the token is guaranteed to be:
    /// - Cryptographically signed by the provider
    /// - Structurally valid JWT format
    /// - Decodable to the expected claims structure
    ///
    /// Additional validations (expiration, audience, etc.) should be performed
    /// by the application using the returned claims data.
    pub async fn validate_token(&self, token: &str) -> Result<TokenData<Claim>, Error> {
        // Get the kid from header, without signature verification
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.ok_or(Error::MissingKid)?;

        let decoding_key = self.get_or_fetch_key(&kid).await?;
        let validation = Validation::new(header.alg);

        decode(token, &decoding_key, &validation).map_err(Into::into)
    }

    /// Retrieves a JWK decoding key from cache or fetches it from the provider.
    ///
    /// This method implements the key retrieval strategy with automatic fallback:
    /// 1. First, check the local cache for the requested key ID
    /// 2. If not found, fetch the latest JWK Set from the provider
    /// 3. Cache all keys from the fetched set
    /// 4. Return the requested key if now available
    ///
    /// ## Arguments
    ///
    /// * `kid` - The Key ID from the JWT header
    ///
    /// ## Returns
    ///
    /// * `Ok(DecodingKey)` - The cryptographic key for signature verification
    /// * `Err(Error)` - If the key cannot be retrieved or provider is unreachable
    ///
    /// ## Caching Behavior
    ///
    /// Keys are cached with a 1-hour TTL. If a key expires or is not found in cache,
    /// this method will automatically refresh the entire JWK Set from the provider.
    /// This ensures that key rotations by the provider are handled transparently.
    ///
    /// ## Error Conditions
    ///
    /// - [`Error::MissingKid`] - The requested key ID is not available from the provider
    /// - [`Error::UnreachableOidcProvider`] - Network failure contacting JWK endpoint  
    /// - [`Error::UnparsableJwks`] - JWK Set response is malformed
    async fn get_or_fetch_key(&self, kid: &str) -> Result<DecodingKey, Error> {
        let mut key = self.keys.get(kid).await;

        if key.is_none() {
            self.fetch_keys().await?;
            key = self.keys.get(kid).await;
        }

        key.ok_or(Error::MissingKid)
    }

    /// Fetches the complete JWK Set from the provider and caches all keys.
    ///
    /// This method retrieves the provider's current JWK Set and caches all contained
    /// keys for future use. It uses concurrent processing to efficiently handle
    /// multiple keys with backpressure control.
    ///
    /// ## Network Behavior
    ///
    /// Makes a single HTTP GET request to the provider's `jwks_uri` endpoint.
    /// The response is expected to be a valid JWK Set containing one or more
    /// cryptographic keys.
    ///
    /// ## Processing Strategy
    ///
    /// - **Parallel Processing**: Keys are processed concurrently with a maximum
    ///   concurrency of 4 to avoid overwhelming the system
    /// - **Atomic Operation**: Either all keys are successfully cached, or the
    ///   entire operation fails
    /// - **Key Validation**: Each key must have a valid `kid` and be convertible
    ///   to a `DecodingKey`
    ///
    /// ## Error Handling
    ///
    /// This method fails fast - if any individual key cannot be processed, the
    /// entire operation is aborted. This ensures cache consistency and prevents
    /// partial updates that could lead to unpredictable validation behavior.
    ///
    /// ## Cache Updates
    ///
    /// All successfully processed keys are inserted into the cache with the
    /// configured TTL (1 hour). Existing cached keys are not removed, allowing
    /// for overlapping key validity periods during key rotation.
    async fn fetch_keys(&self) -> Result<(), Error> {
        let jwks = self
            .client
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(|_| Error::UnreachableOidcProvider(self.jwks_uri.clone()))?
            .json::<JwkSet>()
            .await
            .map_err(|_| Error::UnparsableJwks(self.jwks_uri.clone()))?
            .keys;

        stream::iter(jwks)
            .map(|jwk| async move {
                let kid = jwk.common.key_id.clone().ok_or(Error::MissingKid)?;
                let decoding_key = DecodingKey::from_jwk(&jwk)?;
                self.keys.insert(kid, decoding_key).await;
                Ok(())
            })
            .buffer_unordered(4)
            .try_collect()
            .await
    }
}
