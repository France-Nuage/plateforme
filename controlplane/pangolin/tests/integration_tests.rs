//! Integration tests for the Pangolin API client.
//!
//! These tests run against a real Pangolin instance.
//! Required environment variables:
//! - PANGOLIN_API_URL: The Pangolin API base URL (e.g., http://localhost:3001)
//! - PANGOLIN_API_KEY: A valid Pangolin API key with appropriate permissions
//! - PANGOLIN_ORG_ID: The organization slug/ID for testing
//!
//! To run these tests:
//! 1. Start Pangolin via docker-compose
//! 2. Create a test organization and API key
//! 3. Set the required environment variables
//! 4. Run: cargo test --package pangolin --test integration_tests

use std::env;

/// Test configuration loaded from environment variables.
struct TestConfig {
    api_url: String,
    api_key: String,
    org_id: String,
}

impl TestConfig {
    /// Attempts to load test configuration from environment variables.
    /// Returns None if required variables are not set.
    fn from_env() -> Option<Self> {
        let api_url = env::var("PANGOLIN_API_URL").ok()?;
        let api_key = env::var("PANGOLIN_API_KEY").ok()?;
        let org_id = env::var("PANGOLIN_ORG_ID").ok()?;

        Some(Self {
            api_url,
            api_key,
            org_id,
        })
    }
}

/// Helper to skip tests when Pangolin is not configured.
macro_rules! require_pangolin {
    () => {
        match TestConfig::from_env() {
            Some(config) => config,
            None => {
                eprintln!(
                    "Skipping test: Pangolin not configured. \
                     Set PANGOLIN_API_URL, PANGOLIN_API_KEY, and PANGOLIN_ORG_ID."
                );
                return;
            }
        }
    };
}

mod list_users {
    use super::*;
    use pangolin::api::list_users;

    #[tokio::test]
    async fn test_list_users_returns_users() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = list_users(&config.api_url, &client, &config.api_key, &config.org_id).await;

        assert!(result.is_ok(), "Failed to list users: {:?}", result.err());
        let response = result.unwrap();
        assert!(response.total >= 0, "Total should be non-negative");
    }

    #[tokio::test]
    async fn test_list_users_unauthorized_returns_error() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = list_users(&config.api_url, &client, "invalid-api-key", &config.org_id).await;

        assert!(result.is_err(), "Should fail with invalid API key");
    }
}

mod create_invite {
    use super::*;
    use pangolin::api::create_invite;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_create_invite_success() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        // Generate unique email to avoid conflicts
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_email = format!("test-{}@example.com", timestamp);

        let result = create_invite(
            &config.api_url,
            &client,
            &config.api_key,
            &config.org_id,
            &test_email,
            "member", // Default role
            false,    // Don't send email in tests
            Some(24), // 24 hour validity
        )
        .await;

        assert!(
            result.is_ok(),
            "Failed to create invite: {:?}",
            result.err()
        );
        let response = result.unwrap();
        assert!(
            !response.invite_id.is_empty(),
            "Invite ID should not be empty"
        );
        assert!(!response.token.is_empty(), "Token should not be empty");
    }

    #[tokio::test]
    async fn test_create_invite_unauthorized_returns_error() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = create_invite(
            &config.api_url,
            &client,
            "invalid-api-key",
            &config.org_id,
            "test@example.com",
            "member",
            false,
            None,
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid API key");
    }
}

mod update_user {
    use super::*;
    use pangolin::api::update_user;

    #[tokio::test]
    async fn test_update_user_unauthorized_returns_error() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = update_user(
            &config.api_url,
            &client,
            "invalid-api-key",
            &config.org_id,
            "some-user-id",
            Some("member"),
            None,
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid API key");
    }

    #[tokio::test]
    async fn test_update_nonexistent_user_returns_error() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = update_user(
            &config.api_url,
            &client,
            &config.api_key,
            &config.org_id,
            "nonexistent-user-id",
            Some("member"),
            None,
        )
        .await;

        // Should fail because user doesn't exist
        assert!(result.is_err(), "Should fail for nonexistent user");
    }
}

mod remove_user {
    use super::*;
    use pangolin::api::remove_user;

    #[tokio::test]
    async fn test_remove_user_unauthorized_returns_error() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = remove_user(
            &config.api_url,
            &client,
            "invalid-api-key",
            &config.org_id,
            "some-user-id",
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid API key");
    }

    #[tokio::test]
    async fn test_remove_nonexistent_user_returns_error() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        let result = remove_user(
            &config.api_url,
            &client,
            &config.api_key,
            &config.org_id,
            "nonexistent-user-id",
        )
        .await;

        // Should fail because user doesn't exist
        assert!(result.is_err(), "Should fail for nonexistent user");
    }
}

/// Full lifecycle test: create invite, verify it exists via list.
mod lifecycle {
    use super::*;
    use pangolin::api::{create_invite, list_users};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_invite_lifecycle() {
        let config = require_pangolin!();
        let client = reqwest::Client::new();

        // Get initial user count
        let initial = list_users(&config.api_url, &client, &config.api_key, &config.org_id)
            .await
            .expect("Failed to get initial user list");
        let initial_count = initial.total;

        // Create an invite
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_email = format!("lifecycle-test-{}@example.com", timestamp);

        let invite = create_invite(
            &config.api_url,
            &client,
            &config.api_key,
            &config.org_id,
            &test_email,
            "member",
            false,
            Some(1), // 1 hour validity
        )
        .await
        .expect("Failed to create invite");

        assert!(!invite.invite_id.is_empty());
        assert!(!invite.token.is_empty());

        // Note: The user won't appear in list_users until they accept the invite
        // So we just verify the invite was created successfully

        // Verify user list is still accessible
        let after = list_users(&config.api_url, &client, &config.api_key, &config.org_id)
            .await
            .expect("Failed to get user list after invite");

        // User count should remain the same (invite not yet accepted)
        assert_eq!(
            after.total, initial_count,
            "User count should not change until invite is accepted"
        );
    }
}
