//! Application configuration
//!
//! Loads application configuration from environment variables including SpiceDB
//! connection details and database URL. Use `Config::from_env()` to initialize
//! from environment, returning an error if required variables are missing.

use std::env;

use crate::Error;

pub struct Config {
    pub auth_server_url: String,
    pub auth_server_token: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            auth_server_url: read_env_var("SPICEDB_URL")?,
            auth_server_token: read_env_var("SPICEDB_GRPC_PRESHARED_KEY")?,
            database_url: read_env_var("DATABASE_URL")?,
        })
    }
}

fn read_env_var(var: &str) -> Result<String, Error> {
    env::var(var).map_err(|_| Error::MissingEnvVar(var.to_string()))
}
