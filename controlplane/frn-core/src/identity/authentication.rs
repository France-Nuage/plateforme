// use crate::identity::Error;
// use futures::{StreamExt, TryStreamExt, stream};
// use jsonwebtoken::{DecodingKey, TokenData, Validation, jwk::JwkSet};
// use moka::future::Cache;
// use std::time::Duration;
//
// const JWK_CACHE_MAX_CAPACITY: u64 = 200;
// const JWK_CACHE_TTL: u64 = 3600;
//
// pub struct OpenidConfiguration {
//     pub jwks_uri: String,
// }
//
// pub struct Claim {
//     /// Expiration time (seconds since Unix epoch).
//     exp: String,
//
//     /// Issued at (seconds since Unix epoch).
//     iat: String,
//
//     /// JWT ID (unique identifier for this token)
//     jti: String,
//
//     /// Issuer (who created and signed this token).
//     iss: String,
//
//     /// Type of token.
//     typ: String,
//
//     /// Authorized party (the party to which this token was issued).
//     azp: String,
//
//     /// Session ID (String identifier for a Session).
//     sid: String,
//
//     /// Authentication context class.
//     acr: String,
//
//     ///
//     email: String,
// }
//
// pub struct IdentityProvider {
//     client: reqwest::Client,
//     config: OpenidConfiguration,
//     keys: Cache<String, DecodingKey>,
// }
//
// impl IdentityProvider {
//     pub async fn discover(client: reqwest::Client, url: &str) -> Result<Self, Error> {
//         let config: OpenidConfiguration = client
//             .get(url)
//             .send()
//             .await
//             .map_err(|_| Error::UnreachableOidcProvider(url.to_owned()))?
//             .json()
//             .await
//             .inspect_err(|err| println!("error: {:#?}", err))
//             .map_err(|_| Error::UnparsableOidcMetadata(url.to_owned()))?;
//
//         Ok(Self {
//             client,
//             config,
//             keys: Cache::builder()
//                 .max_capacity(JWK_CACHE_MAX_CAPACITY)
//                 .time_to_live(Duration::from_secs(JWK_CACHE_TTL))
//                 .build(),
//         })
//     }
//
//     /// Validates a JWT token and extracts its claims.
//     pub async fn validate(&self, token: String) -> Result<TokenData<Claim>, Error> {
//         let header = jsonwebtoken::decode_header(token)?;
//         let kid = header.kid.ok_or(Error::MissingKid)?;
//
//         let decoding_key = self.get_or_fetch_key(&kid).await?;
//         let mut validation = Validation::new(header.alg);
//
//         jsonwebtoken::decode(token, &decoding_key, &validation).map_err(Into::into)
//     }
//
//     /// Retrieves a JWK decoding key from cache or fetches it from the provider.
//     async fn get_or_fetch_key(&self, kid: &str) -> Result<DecodingKey, Error> {
//         // attempt to get the key from cache
//         let mut key = self.keys.get(kid).await;
//
//         // if there is a cache miss, fetch keys from the provider and update the cache
//         if key.is_none() {
//             let keys = self.fetch_keys().await?;
//             for (kid, decoding_key) in keys {
//                 self.keys.insert(kid, decoding_key).await;
//             }
//             key = self.keys.get(kid).await;
//         }
//
//         key.ok_or(Error::MissingKid)
//     }
//
//     /// Fetches the complete JWK Set from the provider and caches all keys.
//     async fn fetch_keys(&self) -> Result<Vec<(String, DecodingKey)>, Error> {
//         let jwks = self
//             .client
//             .get(&self.config.jwks_uri)
//             .send()
//             .await
//             .map_err(|_| Error::UnreachableOidcProvider(self.config.jwks_uri.clone()))?
//             .json::<JwkSet>()
//             .await
//             .map_err(|_| Error::UnparsableJwks(self.config.jwks_uri.clone()))?
//             .keys;
//
//         stream::iter(jwks)
//             .map(|jwk| async move {
//                 let kid = jwk.common.key_id.clone().ok_or(Error::MissingKid)?;
//                 let decoding_key = DecodingKey::from_jwk(&jwk)?;
//                 // self.keys.insert(kid, decoding_key).await;
//                 Ok::<(String, DecodingKey), Error>((kid, decoding_key))
//             })
//             .buffer_unordered(4)
//             .try_collect()
//             .await
//     }
// }
