//! Integration tests for the IAM crate.

#[cfg(test)]
mod tests {
    use crate::{
        auth::{AuthConfig, AuthProvider, TokenInfo},
        context::{RequestContext, RequestContextExt},
        error::{AuthError, AuthResult},
        interceptor::AuthInterceptor,
        user::UserInfo,
    };
    use async_trait::async_trait;
    use mockall::{mock, predicate::*};
    use serde_json::json;
    use tonic::{Request, Status};
    use tonic::service::Interceptor;

    // Mock AuthProvider for testing
    mock! {
        pub AuthProvider {}

        #[async_trait]
        impl AuthProvider for AuthProvider {
            async fn validate_token(&self, token: &str) -> AuthResult<UserInfo>;
            async fn refresh_token(&self, refresh_token: &str) -> AuthResult<TokenInfo>;
            fn authorize_url<'a>(&self, state: &str, scopes: &[&'a str]) -> String;
            async fn exchange_code(&self, code: &str) -> AuthResult<TokenInfo>;
            async fn get_user_info(&self, token: &str) -> AuthResult<UserInfo>;
        }

        impl Clone for AuthProvider {
            fn clone(&self) -> Self;
        }
    }

    #[test]
    fn test_user_info_builder() {
        let user_info = UserInfo::new("123", "user@example.com", true)
            .with_name("Test User")
            .with_given_name("Test")
            .with_family_name("User")
            .with_picture("https://example.com/pic.jpg")
            .with_locale("en-US")
            .with_claim("custom_claim", json!("custom_value"));

        assert_eq!(user_info.id, "123");
        assert_eq!(user_info.email, "user@example.com");
        assert_eq!(user_info.email_verified, true);
        assert_eq!(user_info.name, Some("Test User".to_string()));
        assert_eq!(user_info.given_name, Some("Test".to_string()));
        assert_eq!(user_info.family_name, Some("User".to_string()));
        assert_eq!(user_info.picture, Some("https://example.com/pic.jpg".to_string()));
        assert_eq!(user_info.locale, Some("en-US".to_string()));
        
        let custom_claim: String = user_info.get_claim("custom_claim").unwrap();
        assert_eq!(custom_claim, "custom_value");
    }

    #[test]
    fn test_auth_config() {
        let config = AuthConfig::new("client_id", "client_secret", "https://example.com/redirect")
            .with_param("param1", "value1")
            .with_param("param2", "value2");

        assert_eq!(config.client_id, "client_id");
        assert_eq!(config.client_secret, "client_secret");
        assert_eq!(config.redirect_uri, "https://example.com/redirect");
        assert_eq!(config.extra_params.get("param1"), Some(&"value1".to_string()));
        assert_eq!(config.extra_params.get("param2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_request_context() {
        // Test empty context
        let context = RequestContext::new();
        assert_eq!(context.is_authenticated(), false);
        assert_eq!(context.user_info(), None);

        // Test context with user info
        let user_info = UserInfo::new("123", "user@example.com", true);
        let context = RequestContext::with_user(user_info.clone());
        assert_eq!(context.is_authenticated(), true);
        assert_eq!(context.user_info().unwrap().id, user_info.id);
        assert_eq!(context.user_info().unwrap().email, user_info.email);
    }

    #[tokio::test]
    async fn test_interceptor_with_valid_token() {
        let mut mock_provider = MockAuthProvider::new();
        let user_info = UserInfo::new("123", "user@example.com", true);
        
        mock_provider
            .expect_validate_token()
            .with(eq("valid_token"))
            .returning(move |_| Ok(user_info.clone()));

        let mut interceptor = AuthInterceptor::new(mock_provider);
        
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            "Bearer valid_token".parse().unwrap(),
        );
        
        let request = interceptor.call(request).unwrap();
        
        // Now we need to simulate the service call to trigger token validation
        let extensions = request.extensions();
        let context = extensions.get::<RequestContext>();
        
        // At this point, the context is not populated yet because validation is deferred
        assert!(context.is_none());
        
        // We would need to configure the interceptor as part of a service stack to test
        // the full flow, which is beyond the scope of a unit test
    }

    #[tokio::test]
    async fn test_interceptor_with_invalid_token() {
        let mut mock_provider = MockAuthProvider::new();
        
        mock_provider
            .expect_validate_token()
            .with(eq("invalid_token"))
            .returning(|_| Err(AuthError::InvalidToken("Token validation failed".to_string())));

        let mut interceptor = AuthInterceptor::new(mock_provider);
        
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            "Bearer invalid_token".parse().unwrap(),
        );
        
        let _request = interceptor.call(request).unwrap();
        
        // Similar limitations as above, we can't test the full flow here
    }

    #[tokio::test]
    async fn test_interceptor_without_token() {
        let mock_provider = MockAuthProvider::new();
        let mut interceptor = AuthInterceptor::new(mock_provider);
        
        let request = Request::new(());
        
        let result = interceptor.call(request);
        
        // Should fail with unauthenticated error
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_optional_interceptor_without_token() {
        let mock_provider = MockAuthProvider::new();
        let mut interceptor = AuthInterceptor::optional(mock_provider);
        
        let request = Request::new(());
        
        let result = interceptor.call(request);
        
        // Should succeed without a token
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_error_to_status() {
        // Test various error conversions to Status
        let invalid_token = AuthError::InvalidToken("Invalid token".to_string());
        let status = Status::from(invalid_token);
        assert_eq!(status.code(), tonic::Code::Unauthenticated);

        let access_denied = AuthError::AccessDenied("Not authorized".to_string());
        let status = Status::from(access_denied);
        assert_eq!(status.code(), tonic::Code::PermissionDenied);

        let config_error = AuthError::ConfigError("Missing config".to_string());
        let status = Status::from(config_error);
        assert_eq!(status.code(), tonic::Code::FailedPrecondition);

        let internal_error = AuthError::InternalError("Server error".to_string());
        let status = Status::from(internal_error);
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    // Extension trait unit tests
    #[test]
    fn test_request_context_extension() {
        // Test getting context when none exists
        let request: Request<()> = Request::new(());
        let context = request.context();
        assert_eq!(context.is_authenticated(), false);

        // Test setting and getting context
        let user_info = UserInfo::new("123", "user@example.com", true);
        let context = RequestContext::with_user(user_info);
        let request = request.set_context(context);
        
        let retrieved_context = request.context();
        assert_eq!(retrieved_context.is_authenticated(), true);
        assert_eq!(retrieved_context.user_info().unwrap().id, "123");
    }
}