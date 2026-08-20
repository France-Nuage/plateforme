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
            .inspect_err(|err| println!("error: {:#?}", err))
            .map_err(|_| Error::UnparsableOidcMetadata(url.to_owned()))?;

        Ok(Self {
            client,
            config,
            keys: Cache::builder()
                .max_capacity(JWK_CACHE_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(JWK_CACHE_TTL))
                .build(),
        })
    }

    /// Retrieves a JWK decoding key from cache or fetches it from the provider.
    ///
    /// Keys are cached with a 1-hour TTL. Fetches from provider on cache miss.
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

    /// Fetches the JWK Set from the provider and returns all keys.
    ///
    /// Processes keys concurrently with max concurrency of 4.
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
    /// * [`Error::MalformedBearerToken`] - JWT signature invalid, expired, malformed, etc.
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
    /// - Not expired — `exp` is enforced here (`validate_exp = true`)
    ///
    /// Audience (`aud`) is **not** verified here (`validate_aud = false`): the
    /// bearer/user path has no single configured expected audience, so a caller
    /// that has one (the BFF, against its client id) verifies `aud` explicitly on
    /// the returned claims.
    pub async fn validate_token(&self, token: &str) -> Result<TokenData<Claim>, Error> {
        // Get the kid from header, without signature verification
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.ok_or(Error::MissingKid)?;

        let decoding_key = self.get_or_fetch_key(&kid).await?;
        let mut validation = Validation::new(header.alg);
        // Audience is verified by explicit code where an expected value exists
        // (the BFF checks `aud` against its client id); the bearer/user path has
        // no configured expected audience, so leave `aud` to explicit checks.
        validation.validate_aud = false;
        // Reject expired tokens: an expired user id_token presented as a bearer
        // credential must fail closed (service accounts are matched by KEY before
        // this path, so they are unaffected).
        validation.validate_exp = true;

        decode(token, &decoding_key, &validation).map_err(Into::into)
    }

    /// Resolves the subject (`sub`) of an access token from the provider's
    /// UserInfo endpoint (OpenID Connect Core 1.0, Section 5.3).
    ///
    /// The access token's format is not standardised by OIDC, so it is not
    /// guaranteed to carry a `sub` claim (Keycloak includes one, FerrisKey does
    /// not). The UserInfo endpoint, in contrast, is the standard way to obtain
    /// the authenticated subject's claims *from* an access token, and always
    /// identifies the subject. We call it only as a fallback, when the token
    /// itself has no usable `sub`.
    ///
    /// # Errors
    /// Returns [`Error::UserInfoSubjectUnresolved`] when the provider does not
    /// advertise a UserInfo endpoint, it cannot be reached, or its response
    /// carries no non-empty `sub`.
    pub async fn userinfo_subject(&self, access_token: &str) -> Result<String, Error> {
        let endpoint = self
            .config
            .userinfo_endpoint
            .as_deref()
            .ok_or(Error::UserInfoSubjectUnresolved)?;

        #[derive(Deserialize)]
        struct UserInfo {
            sub: Option<String>,
        }

        let user_info: UserInfo = self
            .client
            .get(endpoint)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| Error::UserInfoSubjectUnresolved)?
            .error_for_status()
            .map_err(|_| Error::UserInfoSubjectUnresolved)?
            .json()
            .await
            .map_err(|_| Error::UserInfoSubjectUnresolved)?;

        user_info
            .sub
            .filter(|sub| !sub.is_empty())
            .ok_or(Error::UserInfoSubjectUnresolved)
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
    // Tripwire: the mock backend (deterministic RSA signing key + mock IdP) must
    // never ship in a release binary. This module only compiles when the `mock`
    // feature is on; failing the build when that coincides with a release profile
    // means a successful `cargo build --release` proves mock is disabled in
    // production. (`cargo test --release --all-features` would trip it too — CI
    // runs tests in debug, so it doesn't.)
    #[cfg(not(debug_assertions))]
    compile_error!("the `mock` feature must not be enabled in a release build");

    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::mock::{MOCK_JWK_KID, WithJwks, WithWellKnown};
    use crate::rfc7519::Claim;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use mock_server::MockServer;
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
        pub async fn mock() -> Self {
            let server = MockServer::new().await.with_well_known().with_jwks();
            let openid = OpenID::discover(
                reqwest::Client::new(),
                &format!("{}/.well-known/openid-configuration", &server.url()),
            )
            .await
            .expect("could not initialize mock openid");

            // manually validate a dummy token to force fetching the jwks before the server goes
            // out of scope
            let token = OpenID::token("wile.coyote@acme.org");
            openid
                .validate_token(&token)
                .await
                .expect("could not validate token");

            openid
        }

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
                // A conformant OIDC user token carries a verified email and a
                // subject; the control plane requires both (unverified email or
                // missing subject fails closed), so the mock must supply them.
                email_verified: Some(true),
                sub: Some(format!("mock-subject-{email}")),
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

        /// Signs an arbitrary claims object as an RS256 JWT with the
        /// deterministic mock key.
        ///
        /// Unlike [`OpenID::token`], this accepts a full claims payload, so
        /// tests can mint id_tokens carrying `iss`, `aud`, `nonce`, profile
        /// claims, etc. — everything the confidential-client BFF validates. The
        /// signature is verifiable by an `OpenID` configured against the same
        /// mock JWK Set.
        ///
        /// # Security Note
        ///
        /// Test-only, exactly like [`OpenID::token`]. Never use in production.
        pub fn sign_claims(claims: &serde_json::Value) -> String {
            let (private_key, _) = Self::rsa();

            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(MOCK_JWK_KID.to_owned());

            let pem = private_key
                .to_pkcs8_pem(LineEnding::LF)
                .expect("could not create the pem");

            let e = EncodingKey::from_rsa_pem(pem.as_bytes())
                .expect("could not create the encoding key");

            encode(&header, claims, &e).expect("could not encode token")
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
    #[serde(rename = "issuer")]
    pub _issuer: String,

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

    /// RECOMMENDED. URL of the OP's UserInfo Endpoint. Present when the provider
    /// supports it (OpenID Connect Core 1.0, Section 5.3). We use it to resolve
    /// the subject from an access token when the token itself does not carry a
    /// `sub` claim — the access token's format is not standardised, so `sub` is
    /// not guaranteed in it, whereas the UserInfo response always identifies the
    /// subject.
    pub userinfo_endpoint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_fails_when_server_is_unreachable() {
        let oidc_url = "https://anvil.acme/.well-known/openid-configuration".to_owned();
        let result = OpenID::discover(reqwest::Client::new(), &oidc_url).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::UnreachableOidcProvider(_)
        ));
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_discovery_fails_when_the_metadata_is_unparsable() {
        use mock_server::MockServer;

        let server = MockServer::new().await;
        let url = format!("{}/.well-known/openid-configuration", &server.url());

        let result = OpenID::discover(reqwest::Client::new(), &url).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::UnparsableOidcMetadata(_)
        ));
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_discovery_works_with_a_valid_server() {
        use crate::mock::WithWellKnown;
        use mock_server::MockServer;

        let server = MockServer::new().await.with_well_known();
        let url = format!("{}/.well-known/openid-configuration", &server.url());

        let result = OpenID::discover(reqwest::Client::new(), &url).await;

        assert!(result.is_ok());
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_validate_token() {
        let openid = OpenID::mock().await;
        let token = OpenID::token("wile.coyote@acme.org");

        let result = openid.validate_token(&token).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().claims.email.unwrap(),
            "wile.coyote@acme.org"
        );
    }

    /// The subject is resolved from the UserInfo endpoint — the case that lets an
    /// access token without a `sub` claim (e.g. FerrisKey) still authenticate.
    #[tokio::test]
    async fn test_userinfo_resolves_the_subject() {
        use crate::mock::{WithJwks, WithUserInfo, WithWellKnown};
        use mock_server::MockServer;

        let server = MockServer::new()
            .await
            .with_well_known()
            .with_jwks()
            .with_userinfo("subject-42", "roadrunner@acme.org");
        let openid = OpenID::discover(
            reqwest::Client::new(),
            &format!("{}/.well-known/openid-configuration", &server.url()),
        )
        .await
        .expect("could not initialize mock openid");

        let sub = openid
            .userinfo_subject("any-access-token")
            .await
            .expect("userinfo should resolve the subject");

        assert_eq!(sub, "subject-42");
    }

    /// When the provider advertises no UserInfo endpoint, resolving a subject
    /// from it fails closed rather than panicking.
    #[tokio::test]
    async fn test_userinfo_without_an_endpoint_fails_closed() {
        use mock_server::MockServer;

        // A discovery document without `userinfo_endpoint`.
        let mut server = MockServer::new().await;
        let base = server.url();
        let body = serde_json::json!({
            "issuer": base,
            "jwks_uri": format!("{base}/oauth/discovery/keys"),
        })
        .to_string();
        let mock = server
            .server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/.well-known/openid-configuration$".to_string()),
            )
            .with_body(body)
            .create();
        server.mocks.push(mock);

        let openid = OpenID::discover(
            reqwest::Client::new(),
            &format!("{}/.well-known/openid-configuration", &server.url()),
        )
        .await
        .expect("discovery should still succeed without a userinfo endpoint");

        assert!(matches!(
            openid.userinfo_subject("any-access-token").await,
            Err(Error::UserInfoSubjectUnresolved)
        ));
    }
}
