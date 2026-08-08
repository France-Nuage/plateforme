mod authentication;
mod error;
mod iam;
mod invitation;
mod principal;
mod service_account;
mod session;
mod user;

pub use error::*;
pub use iam::*;
pub use invitation::*;
pub use principal::*;
pub use service_account::*;
pub use session::*;
pub use user::*;

/// Name of the cookie carrying the BFF session (an encrypted, self-contained
/// [`SessionPayload`], sealed with [`SessionKey`]).
///
/// Single source of truth shared between the control-plane's confidential-client
/// BFF (which sets this cookie) and [`iam::IAM::principal`] (which opens it as a
/// fallback to the `Authorization` header, so browser gRPC-web calls authenticate
/// from the httpOnly session cookie instead of a JS-held bearer token).
pub const SESSION_COOKIE_NAME: &str = "frn_session";
