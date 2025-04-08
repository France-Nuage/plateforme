//! User information models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalized user information across different OAuth2 providers.
///
/// This structure contains the core user identity information extracted from 
/// various OAuth2 providers in a normalized format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    /// Unique identifier for the user.
    pub id: String,
    
    /// User's email address.
    pub email: String,
    
    /// Whether the email has been verified.
    pub email_verified: bool,
    
    /// User's display name.
    pub name: Option<String>,
    
    /// User's given (first) name.
    pub given_name: Option<String>,
    
    /// User's family (last) name.
    pub family_name: Option<String>,
    
    /// URL to the user's profile picture.
    pub picture: Option<String>,
    
    /// User's locale preference.
    pub locale: Option<String>,
    
    /// Additional claims from the provider that aren't covered by the standard fields.
    pub additional_claims: HashMap<String, serde_json::Value>,
}

impl UserInfo {
    /// Creates a new UserInfo instance with minimal required fields.
    pub fn new(id: impl Into<String>, email: impl Into<String>, email_verified: bool) -> Self {
        UserInfo {
            id: id.into(),
            email: email.into(),
            email_verified,
            name: None,
            given_name: None,
            family_name: None,
            picture: None,
            locale: None,
            additional_claims: HashMap::new(),
        }
    }

    /// Sets the user's name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the user's given name.
    pub fn with_given_name(mut self, given_name: impl Into<String>) -> Self {
        self.given_name = Some(given_name.into());
        self
    }

    /// Sets the user's family name.
    pub fn with_family_name(mut self, family_name: impl Into<String>) -> Self {
        self.family_name = Some(family_name.into());
        self
    }

    /// Sets the URL to the user's profile picture.
    pub fn with_picture(mut self, picture: impl Into<String>) -> Self {
        self.picture = Some(picture.into());
        self
    }

    /// Sets the user's locale preference.
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    /// Adds an additional claim to the user info.
    pub fn with_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.additional_claims.insert(key.into(), value);
        self
    }

    /// Get a specific additional claim as a typed value.
    pub fn get_claim<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.additional_claims
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
}