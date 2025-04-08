//! Google OAuth2 provider implementation.

use crate::auth::{AuthConfig, AuthProvider, TokenInfo};
use crate::error::{AuthError, AuthResult};
use crate::user::UserInfo;
use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

// Google OAuth2 endpoints
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const GOOGLE_CERTS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

/// Google Auth Provider implementation.
///
/// This provider implements the OAuth2 flow for Google, including token
/// validation, refresh, and user info retrieval.
#[derive(Debug, Clone)]
pub struct GoogleAuthProvider {
    config: AuthConfig,
    http_client: Arc<Client>,
}

/// Claims in a Google ID token.
#[derive(Debug, Serialize, Deserialize)]
struct GoogleIdTokenClaims {
    /// Subject (user ID)
    sub: String,
    /// Email address
    email: String,
    /// Whether the email is verified
    email_verified: bool,
    /// Name
    #[serde(default)]
    name: Option<String>,
    /// Given name
    #[serde(default)]
    given_name: Option<String>,
    /// Family name
    #[serde(default)]
    family_name: Option<String>,
    /// Profile picture URL
    #[serde(default)]
    picture: Option<String>,
    /// Locale
    #[serde(default)]
    locale: Option<String>,
    /// Issuer
    iss: String,
    /// Audience
    aud: String,
    /// Issued at
    iat: i64,
    /// Expiration time
    exp: i64,
}

/// Response from the Google token endpoint.
#[derive(Debug, Serialize, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    token_type: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
}

/// Response from the Google userinfo endpoint.
#[derive(Debug, Serialize, Deserialize)]
struct GoogleUserInfoResponse {
    sub: String,
    email: String,
    email_verified: bool,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    given_name: Option<String>,
    #[serde(default)]
    family_name: Option<String>,
    #[serde(default)]
    picture: Option<String>,
    #[serde(default)]
    locale: Option<String>,
    #[serde(flatten)]
    additional_claims: HashMap<String, serde_json::Value>,
}

impl GoogleAuthProvider {
    /// Creates a new GoogleAuthProvider with the specified configuration.
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            http_client: Arc::new(Client::new()),
        }
    }

    /// Creates a new GoogleAuthProvider with configuration from environment variables.
    pub fn from_env() -> AuthResult<Self> {
        let config = AuthConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Validates a Google ID token and returns the claims if valid.
    async fn validate_id_token(&self, id_token: &str) -> AuthResult<GoogleIdTokenClaims> {
        // Get the JWT header to determine the key ID
        let header = decode_header(id_token)
            .map_err(|e| AuthError::InvalidToken(format!("Invalid token header: {}", e)))?;

        let kid = header.kid.ok_or_else(|| {
            AuthError::InvalidToken("Token header missing 'kid' claim".to_string())
        })?;

        // Fetch the JWKs from Google
        let jwks_response = self
            .http_client
            .get(GOOGLE_CERTS_URL)
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        let jwks: serde_json::Value = jwks_response
            .json()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        // Find the key with matching key ID
        let jwk = jwks["keys"]
            .as_array()
            .and_then(|keys| {
                keys.iter().find(|key| {
                    key["kid"].as_str().map_or(false, |k| k == kid)
                })
            })
            .ok_or_else(|| AuthError::InvalidToken("Key ID not found in JWKs".to_string()))?;

        // Extract the modulus and exponent for RSA keys
        let n = jwk["n"]
            .as_str()
            .ok_or_else(|| AuthError::InvalidToken("Missing modulus in JWK".to_string()))?;
        let e = jwk["e"]
            .as_str()
            .ok_or_else(|| AuthError::InvalidToken("Missing exponent in JWK".to_string()))?;

        // Create a decoding key
        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| AuthError::InvalidToken(format!("Invalid RSA components: {}", e)))?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.client_id]);
        validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
        
        // Validate the token
        let token_data = decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(format!("Token validation failed: {}", e)))?;

        // Check if the token has expired
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AuthError::TokenExpired);
        }

        Ok(token_data.claims)
    }

    /// Converts Google user info to the common UserInfo format.
    fn convert_user_info(google_user: GoogleUserInfoResponse) -> UserInfo {
        let mut user_info = UserInfo::new(
            google_user.sub,
            google_user.email,
            google_user.email_verified,
        );

        if let Some(name) = google_user.name {
            user_info = user_info.with_name(name);
        }

        if let Some(given_name) = google_user.given_name {
            user_info = user_info.with_given_name(given_name);
        }

        if let Some(family_name) = google_user.family_name {
            user_info = user_info.with_family_name(family_name);
        }

        if let Some(picture) = google_user.picture {
            user_info = user_info.with_picture(picture);
        }

        if let Some(locale) = google_user.locale {
            user_info = user_info.with_locale(locale);
        }

        // Add additional claims
        for (key, value) in google_user.additional_claims {
            // Skip fields we've already processed
            if !["sub", "email", "email_verified", "name", "given_name", 
                 "family_name", "picture", "locale"].contains(&key.as_str()) {
                user_info = user_info.with_claim(key, value);
            }
        }

        user_info
    }

    /// Converts token claims to the common UserInfo format.
    fn convert_token_claims(claims: GoogleIdTokenClaims) -> UserInfo {
        let mut user_info = UserInfo::new(
            claims.sub,
            claims.email,
            claims.email_verified,
        );

        if let Some(name) = claims.name {
            user_info = user_info.with_name(name);
        }

        if let Some(given_name) = claims.given_name {
            user_info = user_info.with_given_name(given_name);
        }

        if let Some(family_name) = claims.family_name {
            user_info = user_info.with_family_name(family_name);
        }

        if let Some(picture) = claims.picture {
            user_info = user_info.with_picture(picture);
        }

        if let Some(locale) = claims.locale {
            user_info = user_info.with_locale(locale);
        }

        user_info
    }
}

#[async_trait]
impl AuthProvider for GoogleAuthProvider {
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo> {
        debug!("Validating Google token");

        // Check if it's an ID token (JWT format)
        if token.split('.').count() == 3 {
            // Validate the ID token
            let claims = self.validate_id_token(token).await?;
            return Ok(Self::convert_token_claims(claims));
        }

        // If it's an access token, use it to get user info
        self.get_user_info(token).await
    }

    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<TokenInfo> {
        debug!("Refreshing Google token");

        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::ProviderError(format!(
                "Token refresh failed: HTTP {} - {}",
                status, error_text
            )));
        }

        let token_response: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        Ok(TokenInfo {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            refresh_token: token_response.refresh_token,
            expires_in: token_response.expires_in,
            scope: token_response.scope,
            id_token: token_response.id_token,
        })
    }

    fn authorize_url<'a>(&self, state: &str, scopes: &[&'a str]) -> String {
        let scopes_str = scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            GOOGLE_AUTH_URL,
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&scopes_str),
            urlencoding::encode(state)
        )
    }

    async fn exchange_code(&self, code: &str) -> AuthResult<TokenInfo> {
        debug!("Exchanging code for Google token");

        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::ProviderError(format!(
                "Code exchange failed: HTTP {} - {}",
                status, error_text
            )));
        }

        let token_response: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        Ok(TokenInfo {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            refresh_token: token_response.refresh_token,
            expires_in: token_response.expires_in,
            scope: token_response.scope,
            id_token: token_response.id_token,
        })
    }

    async fn get_user_info(&self, token: &str) -> AuthResult<UserInfo> {
        debug!("Getting Google user info");

        let response = self
            .http_client
            .get(GOOGLE_USERINFO_URL)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::ProviderError(format!(
                "Failed to get user info: HTTP {} - {}",
                status, error_text
            )));
        }

        let user_info: GoogleUserInfoResponse = response
            .json()
            .await
            .map_err(|e| AuthError::HttpError(e))?;

        Ok(Self::convert_user_info(user_info))
    }
}

// We'll implement more detailed tests in the main tests module