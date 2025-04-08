//! Basic example of using the IAM package for OAuth2 authentication in a gRPC service.
//! 
//! This example demonstrates how to use the IAM package with Google OAuth2 authentication.
//! In a real implementation, you would generate proper gRPC code from protobuf definitions.

use iam::{
    auth::AuthConfig,
    context::RequestContextExt,
    interceptor::AuthInterceptor,
    providers::google::GoogleAuthProvider,
};
use std::error::Error;
use tonic::{service::Interceptor, Request, Response, Status};

/// Example function showing how to use the IAM package
fn example_usage() -> Result<(), Box<dyn Error>> {
    // Configure the auth provider
    let auth_config = AuthConfig::new(
        "your-google-client-id",  // Replace with your client ID
        "your-google-client-secret", // Replace with your client secret
        "http://localhost:8080/oauth/callback", // Replace with your redirect URI
    );
    
    // Create the Google auth provider
    let auth_provider = GoogleAuthProvider::new(auth_config);
    
    // Create an auth interceptor that requires authentication
    let required_auth = AuthInterceptor::new(auth_provider.clone());
    
    // Or, create an auth interceptor that makes authentication optional
    let optional_auth = AuthInterceptor::optional(auth_provider);
    
    println!("Auth interceptors created successfully");
    
    // In a real implementation, you would use the interceptor with tonic like this:
    // 
    // Server::builder()
    //     .add_service(
    //         YourServiceServer::new(YourServiceImpl::default())
    //             .with_interceptor(auth_interceptor)
    //     )
    //     .serve(...)
    
    Ok(())
}

/// Example of how to access user information in a gRPC service method
fn example_service_method(request: Request<()>) -> Result<Response<()>, Status> {
    // Get the request context
    let context = request.context();
    
    // Check if the user is authenticated
    if context.is_authenticated() {
        // Get the user information
        let user = context.user_info().unwrap();
        
        println!("Request from user: {}", user.email);
        println!("User ID: {}", user.id);
        
        if let Some(name) = &user.name {
            println!("User name: {}", name);
        }
        
        // Use the user information to implement your business logic
        // ...
    } else {
        println!("Request from unauthenticated user");
        // Handle unauthenticated requests
        // ...
    }
    
    // Process the request and return a response
    Ok(Response::new(()))
}

fn main() {
    println!("This is a code example and not meant to be run directly.");
    println!("See the code for examples of how to use the IAM package.");
}