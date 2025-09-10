use crate::{Error, rfc7519::Claim};
use futures::{StreamExt, TryStreamExt, stream};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode, jwk::JwkSet};
use moka::future::Cache;
use serde::Deserialize;
use std::{fmt::Debug, time::Duration};

const JWK_CACHE_MAX_CAPACITY: u64 = 200;
const JWK_CACHE_TTL: u64 = 3600;

#[derive(Clone)]
pub struct OpenID {
    client: reqwest::Client,
    config: OpenIDProviderConfiguration,

    /// High-performance cache for JWK decoding keys, keyed by `kid` (Key ID)
    keys: Cache<String, DecodingKey>,
}

impl OpenID {
    pub async fn discover(client: reqwest::Client, url: &str) -> Result<Self, Error> {
        let config: OpenIDProviderConfiguration = client
            .get(url)
            .send()
            .await
            .map_err(|_| Error::UnreachableOidcProvider(url.to_owned()))?
            .json()
            .await
            .map_err(|_| Error::UnparsableOidcMetadata(url.to_owned()))?;

        Ok(Self {
            client,
            config,
            keys: Cache::builder()
                .max_capacity(JWK_CACHE_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(JWK_CACHE_TTL))
                .build(),
            // issuer: config.issuer,
            // jwks_uri: config.jwks_uri,
            // keys: Cache::builder()
            //     .max_capacity(200)
            //     .time_to_live(Duration::from_secs(3600))
            //     .build(),
        })
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
        // attempt to get the key from cache
        let mut key = self.keys.get(kid).await;

        // if there is a cache miss, fetch keys from the provider and update the cache
        if key.is_none() {
            let keys = self.fetch_keys().await?;
            for (kid, decoding_key) in keys {
                self.keys.insert(kid, decoding_key).await;
            }
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
    async fn fetch_keys(&self) -> Result<Vec<(String, DecodingKey)>, Error> {
        let jwks = self
            .client
            .get(&self.config.jwks_uri)
            .send()
            .await
            .map_err(|_| Error::UnreachableOidcProvider(self.config.jwks_uri.clone()))?
            .json::<JwkSet>()
            .await
            .map_err(|_| Error::UnparsableJwks(self.config.jwks_uri.clone()))?
            .keys;

        stream::iter(jwks)
            .map(|jwk| async move {
                let kid = jwk.common.key_id.clone().ok_or(Error::MissingKid)?;
                let decoding_key = DecodingKey::from_jwk(&jwk)?;
                // self.keys.insert(kid, decoding_key).await;
                Ok::<(String, DecodingKey), Error>((kid, decoding_key))
            })
            .buffer_unordered(4)
            .try_collect()
            .await
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
}

impl Debug for OpenID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenID")
            .field("client", &self.client)
            .field("config", &self.config)
            .field("keys", &"[obfuscated]")
            .finish()
    }
}

#[cfg(feature = "mock")]
/// Mock support functionality for JWT testing workflows.
///
/// This module provides utilities for creating and managing RSA key pairs and JWT tokens
/// during testing. It ensures consistent key generation and token creation across different
/// test scenarios while maintaining cryptographic validity.
///
/// ## Key Management
///
/// RSA key pairs are generated once per test session using a seeded random number generator
/// to ensure deterministic behavior. The same keys are used for both token signing and
/// JWK Set creation, enabling end-to-end JWT validation testing.
///
/// ## Token Generation  
///
/// Mock JWT tokens are created with standard claims structure and proper RSA signatures.
/// Generated tokens are valid JWTs that can be validated by the same `OpenID`
/// instance when configured with the corresponding mock server endpoints.
pub mod mock {
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::mock::MOCK_JWK_KID;
    use crate::rfc7519::Claim;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use rand::{SeedableRng, rngs::StdRng};
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};
    use rsa::{RsaPrivateKey, RsaPublicKey};

    /// Global RSA key pair cache for deterministic test key generation.
    ///
    /// Keys are generated once per test session using a fixed seed to ensure
    /// reproducible behavior across test runs. The same keys are shared between
    /// token generation and JWK Set creation.
    static RSA_KEYS: OnceLock<(RsaPrivateKey, RsaPublicKey)> = OnceLock::new();

    impl OpenID {
        /// Retrieves or generates the RSA key pair for JWT testing.
        ///
        /// This method provides access to a static RSA key pair that is generated once
        /// per test session using a deterministic seed. The keys are used for both
        /// JWT token signing and JWK Set creation in mock servers.
        ///
        /// # Returns
        ///
        /// A reference to a tuple containing `(RsaPrivateKey, RsaPublicKey)` that
        /// persists for the lifetime of the test session.
        ///
        /// # Key Properties
        ///
        /// - **Bit Length**: 2048 bits for RSA key generation
        /// - **Deterministic**: Uses fixed seed `[42u8; 32]` for reproducible keys
        /// - **Thread Safe**: Generated once and cached in `OnceLock` for concurrent access
        /// - **Test Isolation**: Keys remain consistent within a test session but are
        ///   regenerated for each new test process
        pub fn rsa() -> &'static (RsaPrivateKey, RsaPublicKey) {
            RSA_KEYS.get_or_init(|| {
                let mut rng = StdRng::from_seed([42u8; 32]);
                let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
                let public_key = RsaPublicKey::from(&private_key);
                (private_key, public_key)
            })
        }

        /// Generates a mock JWT token for testing purposes.
        ///
        /// Creates a properly signed JWT token with standard claims structure that can
        /// be validated by `OpenID` instances configured with mock server endpoints.
        /// The token is signed using the RSA private key from `rsa()` method.
        ///
        /// # Arguments
        ///
        /// * `email` - Email address to include in the JWT claims for user identification
        ///
        /// # Returns
        ///
        /// A base64-encoded JWT token string with the following characteristics:
        /// - **Algorithm**: RS256 (RSA with SHA-256)
        /// - **Key ID**: Uses `MOCK_JWK_KID` for consistent key identification
        /// - **Claims**: Includes email, issued-at, expiration (1 hour), and not-before times
        /// - **Validity**: Token expires 1 hour from generation time
        ///
        /// # Examples
        ///
        /// ```
        /// # #[cfg(feature = "mock")]
        /// # mod wrapper_module {
        /// # use auth::OpenID;
        /// # fn example() {
        /// let token = OpenID::token("user@example.com");
        /// // Token can now be used with mock server validation
        /// # }
        /// # }
        /// ```
        ///
        /// # Security Note
        ///
        /// This method is intended **only for testing** and should never be used in
        /// production code. The private key is deterministically generated and not
        /// cryptographically secure for production use.
        pub fn token(email: &str) -> String {
            let (private_key, _) = Self::rsa();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("could not get system time")
                .as_secs();

            let claim = Claim {
                email: Some(email.to_owned()),
                iat: Some(now),
                exp: Some(now + 3600),
                nbf: Some(now),
                ..Default::default()
            };

            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(MOCK_JWK_KID.to_owned());

            let pem = private_key
                .to_pkcs8_pem(LineEnding::LF)
                .expect("could not create the pem");

            let e = EncodingKey::from_rsa_pem(pem.as_bytes())
                .expect("could not create the encoding key");

            encode(&header, &claim, &e).expect("could not encode token")
        }
    }
}

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
#[derive(Clone, Debug, Deserialize)]
pub struct OpenIDProviderConfiguration {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{WithJwks, WithWellKnown};
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_discovery_fails_when_server_is_unreachable() {
        // Arrange an unreachable url
        let oidc_url = "https://anvil.acme/.well-known/openid-configuration".to_owned();

        // Act the call to the OpenID discover method
        let result = OpenID::discover(reqwest::Client::new(), &oidc_url).await;

        // Assert the result
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::UnreachableOidcProvider(_)
        ));
    }

    #[tokio::test]
    async fn test_discovery_fails_when_the_metadata_is_unparsable() {
        // Arrange a mock server that ooesnt serve valid well-known configuration
        let server = MockServer::new().await;
        let url = format!("{}/.well-known/openid-configuration", &server.url());

        // Act the call to the OpenID discover method
        let result = OpenID::discover(reqwest::Client::new(), &url).await;

        // Assert the result
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::UnparsableOidcMetadata(_)
        ));
    }

    #[tokio::test]
    async fn test_discovery_works_with_a_valid_server() {
        // Arrange a mock oidc server that expose valid metadata
        let server = MockServer::new().await.with_well_known();
        let url = format!("{}/.well-known/openid-configuration", &server.url());

        // Act the call to the OpenID discover method
        let result = OpenID::discover(reqwest::Client::new(), &url).await;

        // Assert the result
        assert!(result.is_ok());
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_validate_token() {
        // Arrange a mock oidc server
        let server = MockServer::new().await.with_well_known().with_jwks();
        let openid = OpenID::discover(
            reqwest::Client::new(),
            &format!("{}/.well-known/openid-configuration", &server.url()),
        )
        .await
        .unwrap();
        let token = OpenID::token("wile.coyote@acme.org");

        // Act the call to the validate_token method
        let result = openid.validate_token(&token).await;

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().claims.email.unwrap(),
            "wile.coyote@acme.org"
        );
    }
}
