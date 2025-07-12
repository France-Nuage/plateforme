use regex::Regex;
use serde::{Deserialize, de::DeserializeOwned};

use super::Problem;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ApiInvalidResponse {
    pub message: String,
    pub errors: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ApiInternalErrorResponse {
    pub message: String,
}

pub trait ApiResponseExt {
    fn to_api_response<T>(
        self,
    ) -> impl std::future::Future<Output = Result<ApiResponse<T>, super::problem::Problem>> + Send
    where
        T: DeserializeOwned;
}

impl ApiResponseExt for Result<reqwest::Response, reqwest::Error> {
    async fn to_api_response<T>(self) -> Result<ApiResponse<T>, super::problem::Problem>
    where
        T: DeserializeOwned,
    {
        let response = self?;
        match response.status() {
            reqwest::StatusCode::OK => response.json::<ApiResponse<T>>().await.map_err(Into::into),
            reqwest::StatusCode::BAD_REQUEST => {
                let response = response.json::<ApiInvalidResponse>().await?;
                Err(super::problem::Problem::Invalid { response })
            }
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                // Regex matching `Problem::MissingAgent` errors
                let missing_agent_rx = Regex::new(r"^No QEMU guest agent configured\n$").unwrap();
                // Regex matching `Problem::VMNotFound` errors
                let vm_not_found_rx = Regex::new(
                    r"^Configuration file 'nodes/.*?/qemu-server/(\d+)\.conf' does not exist\n$",
                )
                .unwrap();
                // Regex matching `Problem::VMNotRunning` errors
                let vm_not_running_rx = Regex::new(r"^VM (\d+) is not running\n$").unwrap();

                let response = response.json::<ApiInternalErrorResponse>().await?;

                match &response.message {
                    // Handle "No QEMU guest agent configured" error
                    message if missing_agent_rx.is_match(message) => Err(Problem::MissingAgent),
                    // Handle "VM Not Found" error
                    message if vm_not_found_rx.is_match(message) => {
                        let id = vm_not_found_rx.captures(message).unwrap()[0]
                            .parse::<u32>()
                            .expect("could not parse vm id to u32");
                        Err(Problem::VMNotFound(id))
                    }
                    // Handle "VM Not Running" error
                    message if vm_not_running_rx.is_match(message) => {
                        let id = vm_not_running_rx.captures(message).unwrap()[1]
                            .parse::<u32>()
                            .expect("could not parse vm id to u32");
                        Err(Problem::VMNotRunning(id))
                    }
                    // Handle other errors
                    _ => Err(Problem::Internal { response }),
                }
            }
            reqwest::StatusCode::FOUND => {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .expect("a 302 response should have a location header")
                    .to_str()
                    .expect("a location header value should be parsable to a string");
                let url = url::Url::parse(location)
                    .expect("a location string should be parsable into a url");
                let host = url.host_str().expect("a Url should have a host");

                if host.ends_with("cloudflareaccess.com") {
                    Err(Problem::GuardedByCloudflare)
                } else {
                    Err(Problem::UnexpectedRedirect(url))
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(Problem::Unauthorized),
            _ => panic!("Unexpected response status: {:?}", response.status()),
        }
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::mock::MockServer;

    pub trait WithApiInternalResponseError {
        fn with_vm_not_running_error(self) -> Self;
        fn with_no_agent_configured(self) -> Self;
    }

    impl WithApiInternalResponseError for MockServer {
        fn with_vm_not_running_error(mut self) -> Self {
            for method in ["DELETE", "GET", "POST", "PATCH", "PUT"].into_iter() {
                let mock = self
                    .server
                    .mock(method, mockito::Matcher::Any)
                    .with_body(r#"{"data":null,"message":"VM 100 is not running\n"}"#)
                    .with_status(500)
                    .create();
                self.mocks.push(mock);
            }

            self
        }

        fn with_no_agent_configured(mut self) -> Self {
            for method in ["DELETE", "GET", "POST", "PATCH", "PUT"].into_iter() {
                let mock = self
                    .server
                    .mock(method, mockito::Matcher::Any)
                    .with_body(r#"{"data":null,"message":"No QEMU guest agent configured\n"}"#)
                    .with_status(500)
                    .create();
                self.mocks.push(mock);
            }

            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{MockServer, WithApiInternalResponseError},
        proxmox_api::cluster_next_id,
    };

    #[tokio::test]
    async fn test_a_missing_agent_error_is_properly_detected() {
        // Arrange a client responding with a "no QEMU guest agent configured"
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_no_agent_configured();

        // Act the call to the function
        let result = cluster_next_id(&server.url(), &client, "").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Problem::MissingAgent));
    }

    #[tokio::test]
    async fn test_a_vm_not_running_error_is_properly_detected() {
        // Arrange a client responding with a "vm not running error"
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_vm_not_running_error();

        // Act the call to the function
        let result = cluster_next_id(&server.url(), &client, "").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Problem::VMNotRunning(100)));
    }
}
