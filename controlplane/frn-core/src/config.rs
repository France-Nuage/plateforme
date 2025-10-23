//! Application configuration
//!
//! Loads application configuration from environment variables including SpiceDB
//! connection details and database URL. Use `Config::from_env()` to initialize
//! from environment, returning an error if required variables are missing.

use std::env;

use crate::Error;

#[derive(Clone)]
pub struct Config {
    pub auth_server_url: String,
    pub auth_server_token: String,
    pub database_url: String,
    pub root_organization: RootOrganization,
}

#[derive(Clone)]
pub struct RootOrganization {
    pub name: String,
    pub service_account_key: Option<String>,
    pub service_account_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            auth_server_url: read_env_var("SPICEDB_URL")?,
            auth_server_token: read_env_var("SPICEDB_GRPC_PRESHARED_KEY")?,
            database_url: read_env_var("DATABASE_URL")?,
            root_organization: RootOrganization {
                name: env::var("ROOT_ORGANIZATION_NAME").unwrap_or("acme".to_owned()),
                service_account_key: env::var("ROOT_SERVICE_ACCOUNT_KEY").ok(),
                service_account_name: env::var("ROOT_SERVICE_ACCOUNT_NAME")
                    .unwrap_or("acme_svc".to_owned()),
            },
        })
    }

    pub fn test() -> Self {
        Self {
            auth_server_url: "".to_owned(),
            auth_server_token: "".to_owned(),
            database_url: "".to_owned(),
            root_organization: RootOrganization {
                name: "".to_owned(),
                service_account_key: None,
                service_account_name: "".to_owned(),
            },
        }
    }
}

fn read_env_var(var: &str) -> Result<String, Error> {
    env::var(var).map_err(|_| Error::MissingEnvVar(var.to_string()))
}
