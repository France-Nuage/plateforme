use subtle::ConstantTimeEq;
use tonic::{Request, Status};

/// Validates the Bearer token from a gRPC request metadata against an expected value
/// using constant-time comparison to prevent timing attacks.
pub fn authenticate_bearer(
    request: &Request<impl Sized>,
    expected_token: &str,
    error_message: &str,
) -> Result<(), Status> {
    let token = request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(t) if t.as_bytes().ct_eq(expected_token.as_bytes()).into() => Ok(()),
        _ => Err(Status::unauthenticated(error_message)),
    }
}
