//! Minimal Prometheus metrics facility for the control plane.
//!
//! The repo had no metrics facility, so this adds the standard `metrics` facade
//! plus a Prometheus recorder exposed at `GET /metrics`. Counters are emitted
//! with the lightweight `metrics::counter!` macro (a no-op until a recorder is
//! installed, which keeps unit tests and metric-less builds panic-free).
//!
//! Auth observability (O1-A) lives here as thin, typed helpers so call sites in
//! [`crate::bff`] never hand-write metric names or leak secrets into labels.
//! Label values are fixed `&'static str` enums — never tokens, emails, or keys.

use std::sync::OnceLock;

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Installs the global Prometheus recorder exactly once (per process) and
/// returns the render handle used by the `/metrics` endpoint.
///
/// Idempotent via [`OnceLock`]: repeated calls (e.g. one server per black-box
/// test in the same process) reuse the first-installed recorder instead of
/// panicking on a double install.
pub fn handle() -> &'static PrometheusHandle {
    HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install the prometheus metrics recorder")
    })
}

/// Renders the current metrics in the Prometheus text exposition format.
pub fn render() -> String {
    handle().render()
}

/// A successful start of the confidential-client login flow (`/auth/login`).
pub fn login() {
    metrics::counter!("auth_login_total").increment(1);
}

/// A rejected `/auth/callback`, labelled by the first failing gate.
pub fn callback_reject(reason: CallbackReject) {
    metrics::counter!("auth_callback_reject_total", "reason" => reason.as_str()).increment(1);
}

/// Outcome of an `/auth/refresh` attempt.
pub fn refresh(result: RefreshResult) {
    metrics::counter!("auth_refresh_total", "result" => result.as_str()).increment(1);
}

/// Why an `/auth/callback` was rejected (label value for `auth_callback_reject_total`).
#[derive(Clone, Copy, Debug)]
pub enum CallbackReject {
    /// CSRF `state` missing or mismatched.
    State,
    /// `nonce` cookie missing or id_token nonce mismatch.
    Nonce,
    /// Upstream failure: the IdP returned an error, no code was returned, or the
    /// authorization-code exchange failed at the token endpoint.
    Exchange,
    /// Token response carried no id_token.
    NoIdToken,
    /// id_token failed signature/iss/aud/exp validation.
    Validation,
}

impl CallbackReject {
    fn as_str(self) -> &'static str {
        match self {
            CallbackReject::State => "state",
            CallbackReject::Nonce => "nonce",
            CallbackReject::Exchange => "exchange",
            CallbackReject::NoIdToken => "no_id_token",
            CallbackReject::Validation => "validation",
        }
    }
}

/// Result of an `/auth/refresh` attempt (label value for `auth_refresh_total`).
#[derive(Clone, Copy, Debug)]
pub enum RefreshResult {
    /// A fresh session was resealed.
    Ok,
    /// The IdP rejected the refresh, or the response was unusable.
    Rejected,
    /// The presented cookie could not be decrypted/parsed.
    DecryptFail,
}

impl RefreshResult {
    fn as_str(self) -> &'static str {
        match self {
            RefreshResult::Ok => "ok",
            RefreshResult::Rejected => "rejected",
            RefreshResult::DecryptFail => "decrypt_fail",
        }
    }
}
