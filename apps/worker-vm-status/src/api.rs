use serde::Deserialize;

pub mod endpoints;
pub mod pagination;

pub use endpoints::*;

/// Define shared behavior for api operations
pub trait ApiOperationQuery {
    /// The base url under which the API is available
    fn base_url(&self) -> String {
        std::env::var("API_URL").unwrap_or(String::from("http://localhost:3333"))
    }

    /// The access_token for authenticating against the API
    fn access_token(&self) -> String {
        std::env::var("API_TOKEN").expect("Missing env var API_TOKEN")
    }
}

/// The API response metadata
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponseMeta {
    pub current_page: u32,
    pub last_page: u32,
}

/// The API response
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub meta: ApiResponseMeta,
    pub data: T,
}
