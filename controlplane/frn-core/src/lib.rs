mod app;
pub mod authorization;
mod config;
mod error;
pub mod identity;
pub mod resourcemanager;

pub use app::App;
pub use config::Config;
pub use error::Error;

// Allow the frn-derive macro to generate code using `::frn_core::...` paths
// that work both when used externally and when used within frn-core itself.
// This solves the circular dependency where frn-derive depends on frn-core,
// and frn-core uses frn-derive macros.
extern crate self as frn_core;
