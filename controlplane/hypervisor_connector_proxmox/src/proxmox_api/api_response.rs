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
    async fn to_api_response<T>(self) -> Result<ApiResponse<T>, super::problem::Problem>
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
                let vm_not_found_rx = Regex::new(
                    r"^Configuration file 'nodes/.*?/qemu-server/(\d+)\.conf' does not exist\n$",
                )
                .unwrap();
                let response = response.json::<ApiInternalErrorResponse>().await?;

                if let Some(vm_id) = vm_not_found_rx.captures(&response.message) {
                    return Err(super::problem::Problem::VMNotFound(
                        vm_id
                            .get(1)
                            .unwrap()
                            .as_str()
                            .parse::<u32>()
                            .expect("could not parse vm id to string"),
                    ));
                }
                Err(super::problem::Problem::Internal { response })
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
