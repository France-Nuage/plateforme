use std::env;

pub struct Config {
    pub auth_server_url: String,
    pub auth_server_token: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            auth_server_url: read_env_var("AUTH_SERVER_URL"),
            auth_server_token: read_env_var("SPICEDB_GRPC_PRESHARED_KEY"),
            database_url: read_env_var("DATABASE_URL"),
        }
    }
}

fn read_env_var(var: &str) -> String {
    env::var(var).unwrap_or_else(|_| panic!("{} must be set", var))
}
