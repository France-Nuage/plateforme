//! API response handling for Pangolin.

use serde::de::DeserializeOwned;

use super::Error;

/// Extension trait for converting reqwest responses to typed results.
pub trait ApiResponseExt {
    /// Converts a reqwest response to a typed result.
    fn to_json<T>(self) -> impl std::future::Future<Output = Result<T, Error>> + Send
    where
        T: DeserializeOwned;

    /// Handles responses that return no content (204).
    fn to_empty(self) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}

impl ApiResponseExt for Result<reqwest::Response, reqwest::Error> {
    async fn to_json<T>(self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response = self?;
        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => response
                .json::<T>()
                .await
                .map_err(|e| Error::UnexpectedResponse(format!("Failed to parse response: {}", e))),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                Err(Error::Unauthorized)
            }
            reqwest::StatusCode::NOT_FOUND => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::NotFound(body))
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::BadRequest(body))
            }
            reqwest::StatusCode::CONFLICT => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::Conflict(body))
            }
            status if status.is_server_error() => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::Internal(body))
            }
            status => Err(Error::UnexpectedResponse(format!(
                "Unexpected status code: {}",
                status
            ))),
        }
    }

    async fn to_empty(self) -> Result<(), Error> {
        let response = self?;
        match response.status() {
            reqwest::StatusCode::NO_CONTENT
            | reqwest::StatusCode::OK
            | reqwest::StatusCode::CREATED => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                Err(Error::Unauthorized)
            }
            reqwest::StatusCode::NOT_FOUND => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::NotFound(body))
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::BadRequest(body))
            }
            reqwest::StatusCode::CONFLICT => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::Conflict(body))
            }
            status if status.is_server_error() => {
                let body = response.text().await.unwrap_or_default();
                Err(Error::Internal(body))
            }
            status => Err(Error::UnexpectedResponse(format!(
                "Unexpected status code: {}",
                status
            ))),
        }
    }
}
