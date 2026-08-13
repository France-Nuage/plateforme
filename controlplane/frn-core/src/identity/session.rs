//! Encrypted, self-contained BFF session cookie.
//!
//! Under the confidential-client flow the browser never holds a token: its
//! session is an **authenticated-encrypted** blob (the [`SESSION_COOKIE_NAME`]
//! cookie) that only the control plane can open. This is the single source of
//! truth for that seal/open, used by BOTH the server crate's BFF (which sets and
//! refreshes the cookie) and [`crate::identity::IAM::principal`] (which opens it
//! to authenticate browser gRPC-web calls).
//!
//! The payload is deliberately minimal — a refresh token plus the short-lived
//! identity — so it stays well under the 4 KB cookie ceiling. The whole (fat)
//! id_token is never stored. Sealing reuses the audited AEAD in [`frn_crypto`]
//! (XChaCha20-Poly1305) with a fixed AAD for domain separation from other
//! ciphertexts (e.g. kubeconfigs).

use std::sync::Arc;

use frn_crypto::Kek;
use serde::{Deserialize, Serialize};

/// Additional authenticated data binding a token to this exact purpose, so a
/// ciphertext produced elsewhere in [`frn_crypto`] can never be replayed as a
/// session cookie (and vice versa).
const SESSION_AAD: &[u8] = b"frn-session-cookie-v1";

/// Deterministic session key for tests only. **Not a secret**: tests run against
/// an isolated database and a fixed value keeps runs reproducible (mirrors the
/// server's `TEST_KUBECONFIG_ENCRYPTION_KEK`).
pub const TEST_SESSION_KEY: [u8; 32] = [24u8; 32];

/// Self-contained identity carried, encrypted, by the `frn_session` cookie.
///
/// `exp` is the **short** access/id-token expiry (unix seconds), independent of
/// the cookie's `Max-Age` (the longer refresh window). The refresh token is
/// redacted from `Debug` so it never leaks into logs or traces.
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionPayload {
    /// OIDC refresh token, used server-side by `/auth/refresh`. Never exposed to
    /// JavaScript, never logged.
    pub refresh_token: String,
    /// OIDC subject identifier (`sub`).
    pub sub: String,
    /// User email — the authoritative identity key on the control plane.
    pub email: String,
    /// Access/id-token expiry (unix seconds): the short lifetime after which the
    /// browser must call `/auth/refresh`.
    pub exp: u64,
}

impl std::fmt::Debug for SessionPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionPayload")
            .field("refresh_token", &"[redacted]")
            .field("sub", &self.sub)
            .field("email", &self.email)
            .field("exp", &self.exp)
            .finish()
    }
}

/// Failure modes of the session seal. None ever carries key or token material,
/// so they are safe to log; every open failure collapses to a single opaque
/// variant (fail closed — no oracle on why decryption failed).
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// The configured `AUTH_COOKIE_KEY` is not valid base64 32-byte material.
    #[error("invalid session cookie key")]
    InvalidKey,

    /// Serialization/encryption of a fresh session failed.
    #[error("failed to seal session cookie")]
    Seal,

    /// The cookie could not be decrypted or parsed (tampered, wrong key,
    /// truncated, or malformed payload).
    #[error("failed to open session cookie")]
    Open,
}

/// The server-side key that seals and opens session cookies.
///
/// Cheaply cloneable (shares the underlying [`Kek`] behind an `Arc`), so it can
/// live inside the cloneable `IAM` and BFF. Never derives a `Debug` that would
/// print the key.
#[derive(Clone)]
pub struct SessionKey {
    kek: Arc<Kek>,
}

impl SessionKey {
    /// Builds the key from a standard-base64 32-byte secret (`AUTH_COOKIE_KEY`).
    pub fn from_base64(encoded: &str) -> Result<Self, SessionError> {
        let kek = Kek::from_base64(encoded).map_err(|_| SessionError::InvalidKey)?;
        Ok(Self { kek: Arc::new(kek) })
    }

    /// Builds the key from raw 32-byte material (tests, deterministic setups).
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            kek: Arc::new(Kek::from_bytes(bytes)),
        }
    }

    /// Seals `payload` into the opaque cookie value.
    pub fn seal(&self, payload: &SessionPayload) -> Result<String, SessionError> {
        let json = serde_json::to_vec(payload).map_err(|_| SessionError::Seal)?;
        frn_crypto::seal(&self.kek, &json, SESSION_AAD).map_err(|_| SessionError::Seal)
    }

    /// Opens (decrypts + parses) a cookie value back into a [`SessionPayload`].
    ///
    /// Any tampering, wrong key, or malformed payload returns
    /// [`SessionError::Open`] — the caller must treat this as unauthenticated.
    pub fn open(&self, cookie: &str) -> Result<SessionPayload, SessionError> {
        let plaintext =
            frn_crypto::open(&self.kek, cookie, SESSION_AAD).map_err(|_| SessionError::Open)?;
        serde_json::from_slice(&plaintext).map_err(|_| SessionError::Open)
    }
}

impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKey").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload() -> SessionPayload {
        SessionPayload {
            refresh_token: "rt-secret".to_owned(),
            sub: "subject-123".to_owned(),
            email: "user@francenuage.fr".to_owned(),
            exp: 1_900_000_000,
        }
    }

    #[test]
    fn seal_then_open_roundtrips() {
        let key = SessionKey::from_bytes(TEST_SESSION_KEY);
        let cookie = key.seal(&payload()).expect("seal");
        let opened = key.open(&cookie).expect("open");

        assert_eq!(opened.email, "user@francenuage.fr");
        assert_eq!(opened.refresh_token, "rt-secret");
        assert_eq!(opened.exp, 1_900_000_000);
    }

    #[test]
    fn the_cookie_is_opaque() {
        let key = SessionKey::from_bytes(TEST_SESSION_KEY);
        let cookie = key.seal(&payload()).expect("seal");

        // Nothing sensitive is readable in the cookie value.
        assert!(!cookie.contains("rt-secret"));
        assert!(!cookie.contains("user@francenuage.fr"));
    }

    #[test]
    fn open_rejects_a_foreign_key() {
        let cookie = SessionKey::from_bytes([1u8; 32])
            .seal(&payload())
            .expect("seal");

        assert!(matches!(
            SessionKey::from_bytes([2u8; 32]).open(&cookie),
            Err(SessionError::Open)
        ));
    }

    #[test]
    fn open_rejects_garbage() {
        let key = SessionKey::from_bytes(TEST_SESSION_KEY);

        assert!(matches!(
            key.open("tampered.garbage"),
            Err(SessionError::Open)
        ));
    }

    #[test]
    fn debug_does_not_leak_the_refresh_token() {
        let debug = format!("{:?}", payload());

        assert!(!debug.contains("rt-secret"));
        assert!(debug.contains("[redacted]"));
    }
}
