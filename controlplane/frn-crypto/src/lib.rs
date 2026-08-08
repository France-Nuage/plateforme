//! Authenticated envelope encryption for secrets stored at rest.
//!
//! Provides reversible, authenticated encryption (AEAD) for sensitive material
//! such as Kubernetes kubeconfigs that must be decrypted later to be used. It
//! implements envelope encryption: a fresh random Data Encryption Key (DEK)
//! encrypts each payload, and the DEK itself is wrapped by a long-lived Key
//! Encryption Key (KEK) that never touches the database. The primitive is
//! XChaCha20-Poly1305 (192-bit random nonce, constant-time, no AES-NI needed).
//!
//! Note: this is *not* hashing. Secrets are recovered in plaintext; there is no
//! salting because that is a one-way password-hashing concept. The per-record
//! random nonce plays the equivalent diversification role here.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Size in bytes of a Key Encryption Key (256-bit).
pub const KEK_SIZE: usize = 32;

/// Size in bytes of an XChaCha20-Poly1305 nonce (192-bit).
pub const NONCE_SIZE: usize = 24;

/// Identifier of the AEAD algorithm, persisted alongside ciphertext so the
/// scheme can evolve without ambiguity.
pub const ALGORITHM: &str = "xchacha20poly1305";

/// Current KEK version. Stored per record to support key rotation: a new KEK
/// bumps this value while old records keep their original version until
/// re-wrapped.
pub const CURRENT_KEY_VERSION: i32 = 1;

/// Errors raised by the envelope encryption layer.
#[derive(Debug, Error)]
pub enum EncryptionError {
    /// The configured KEK did not decode to exactly [`KEK_SIZE`] bytes.
    #[error("invalid key encryption key: expected {KEK_SIZE} bytes, got {0}")]
    InvalidKeyLength(usize),

    /// The base64 KEK material could not be decoded.
    #[error("could not base64-decode the key encryption key")]
    InvalidKeyEncoding,

    /// A stored nonce did not have the expected [`NONCE_SIZE`] length.
    #[error("invalid nonce length: expected {NONCE_SIZE} bytes, got {0}")]
    InvalidNonceLength(usize),

    /// AEAD encryption failed (should not happen for valid keys/inputs).
    #[error("encryption failed")]
    EncryptFailed,

    /// AEAD decryption or authentication failed: wrong key, wrong AAD, or the
    /// ciphertext/nonce was tampered with.
    #[error("decryption failed (authentication error)")]
    DecryptFailed,

    /// The wrapped DEK did not decrypt to exactly [`KEK_SIZE`] bytes.
    #[error("invalid data encryption key length after unwrap: expected {KEK_SIZE} bytes, got {0}")]
    InvalidDekLength(usize),
}

/// A 256-bit Key Encryption Key held in memory. Constructed from the runtime
/// environment (never stored in the database) and used only to wrap/unwrap
/// per-record DEKs.
#[derive(ZeroizeOnDrop)]
pub struct Kek([u8; KEK_SIZE]);

impl Kek {
    /// Builds a KEK from raw 32-byte material.
    pub fn from_bytes(bytes: [u8; KEK_SIZE]) -> Self {
        Self(bytes)
    }

    /// Builds a KEK from a standard-base64 encoded 32-byte secret.
    ///
    /// Returns [`EncryptionError::InvalidKeyEncoding`] when the input is not
    /// valid base64 and [`EncryptionError::InvalidKeyLength`] when the decoded
    /// material is not exactly [`KEK_SIZE`] bytes.
    pub fn from_base64(encoded: &str) -> Result<Self, EncryptionError> {
        let decoded = STANDARD
            .decode(encoded.trim())
            .map_err(|_| EncryptionError::InvalidKeyEncoding)?;
        let bytes: [u8; KEK_SIZE] = decoded
            .as_slice()
            .try_into()
            .map_err(|_| EncryptionError::InvalidKeyLength(decoded.len()))?;
        Ok(Self(bytes))
    }
}

/// Result of envelope-encrypting a payload. Each field maps directly to a
/// database column; nothing here is sensitive once the KEK is kept secret.
#[derive(Clone)]
pub struct EnvelopeCiphertext {
    /// Payload encrypted with the per-record DEK.
    pub ciphertext: Vec<u8>,
    /// 24-byte nonce used to encrypt the payload.
    pub nonce: Vec<u8>,
    /// The DEK, encrypted (wrapped) with the KEK.
    pub dek_ciphertext: Vec<u8>,
    /// 24-byte nonce used to wrap the DEK.
    pub dek_nonce: Vec<u8>,
    /// KEK version used to wrap the DEK.
    pub key_version: i32,
}

impl fmt::Debug for EnvelopeCiphertext {
    /// Redacts the wrapped DEK and ciphertext, exposing only field lengths so
    /// encrypted key material never leaks into logs or traces.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvelopeCiphertext")
            .field("ciphertext_len", &self.ciphertext.len())
            .field("nonce_len", &self.nonce.len())
            .field("dek_ciphertext_len", &self.dek_ciphertext.len())
            .field("dek_nonce_len", &self.dek_nonce.len())
            .field("key_version", &self.key_version)
            .finish()
    }
}

/// Envelope-encrypts `plaintext`.
///
/// Generates a fresh random DEK, encrypts `plaintext` with it, then wraps the
/// DEK with `kek`. `aad` (additional authenticated data, e.g. the record id and
/// key version) is bound to both layers so a ciphertext cannot be moved to a
/// different record without failing authentication. `key_version` is stored to
/// identify which KEK wrapped the DEK.
pub fn encrypt(
    kek: &Kek,
    plaintext: &[u8],
    aad: &[u8],
    key_version: i32,
) -> Result<EnvelopeCiphertext, EncryptionError> {
    let dek = XChaCha20Poly1305::generate_key(&mut OsRng);

    let (ciphertext, nonce) = aead_encrypt(dek.as_slice(), plaintext, aad)?;
    let (dek_ciphertext, dek_nonce) = aead_encrypt(&kek.0, dek.as_slice(), aad)?;

    Ok(EnvelopeCiphertext {
        ciphertext,
        nonce,
        dek_ciphertext,
        dek_nonce,
        key_version,
    })
}

/// Reverses [`encrypt`], returning the original plaintext.
///
/// `aad` must be byte-identical to the value passed to [`encrypt`]. Any
/// mismatch, wrong key, or tampering yields [`EncryptionError::DecryptFailed`].
pub fn decrypt(
    kek: &Kek,
    ciphertext: &EnvelopeCiphertext,
    aad: &[u8],
) -> Result<Vec<u8>, EncryptionError> {
    let dek_bytes = Zeroizing::new(aead_decrypt(
        &kek.0,
        &ciphertext.dek_ciphertext,
        &ciphertext.dek_nonce,
        aad,
    )?);
    let mut dek: [u8; KEK_SIZE] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| EncryptionError::InvalidDekLength(dek_bytes.len()))?;

    let plaintext = aead_decrypt(&dek, &ciphertext.ciphertext, &ciphertext.nonce, aad);
    dek.zeroize();
    plaintext
}

/// Seals `plaintext` into a compact, URL-safe token: `base64url(nonce || ciphertext)`.
///
/// Single-key authenticated encryption (XChaCha20-Poly1305) under `kek`, with
/// `aad` bound for domain separation. Unlike [`encrypt`], there is no envelope
/// (no per-record DEK): the token stays small, which suits a short-lived
/// credential carried in a cookie. The output is opaque and tamper-evident —
/// any wrong key, wrong `aad`, truncation, or bit-flip makes [`open`] fail with
/// [`EncryptionError::DecryptFailed`] (fail closed, never a panic).
pub fn seal(kek: &Kek, plaintext: &[u8], aad: &[u8]) -> Result<String, EncryptionError> {
    let (ciphertext, nonce) = aead_encrypt(&kek.0, plaintext, aad)?;
    let mut blob = Vec::with_capacity(nonce.len() + ciphertext.len());
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(blob))
}

/// Reverses [`seal`], returning the original plaintext.
///
/// `aad` must be byte-identical to the value passed to [`seal`]. Any mismatch,
/// wrong key, truncation, or tampering yields [`EncryptionError::DecryptFailed`].
pub fn open(kek: &Kek, token: &str, aad: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let blob = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| EncryptionError::DecryptFailed)?;
    if blob.len() < NONCE_SIZE {
        return Err(EncryptionError::DecryptFailed);
    }
    let (nonce, ciphertext) = blob.split_at(NONCE_SIZE);
    aead_decrypt(&kek.0, ciphertext, nonce, aad)
}

/// Encrypts `plaintext` with a 256-bit `key`, returning `(ciphertext, nonce)`.
fn aead_encrypt(
    key: &[u8],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
    if key.len() != KEK_SIZE {
        return Err(EncryptionError::InvalidKeyLength(key.len()));
    }
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| EncryptionError::EncryptFailed)?;
    Ok((ciphertext, nonce.to_vec()))
}

/// Decrypts `ciphertext` with a 256-bit `key` and the stored `nonce`.
fn aead_decrypt(
    key: &[u8],
    ciphertext: &[u8],
    nonce: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, EncryptionError> {
    if nonce.len() != NONCE_SIZE {
        return Err(EncryptionError::InvalidNonceLength(nonce.len()));
    }
    if key.len() != KEK_SIZE {
        return Err(EncryptionError::InvalidKeyLength(key.len()));
    }
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = XNonce::from_slice(nonce);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| EncryptionError::DecryptFailed)
}

#[cfg(test)]
mod tests {
    use base64::Engine;

    use super::{CURRENT_KEY_VERSION, EncryptionError, Kek, decrypt, encrypt, open, seal};

    fn test_kek() -> Kek {
        Kek::from_bytes([7u8; 32])
    }

    #[test]
    fn seal_then_open_returns_original_plaintext() {
        let kek = test_kek();
        let token = seal(&kek, b"{\"email\":\"a@b.c\"}", b"session-aad").unwrap();

        assert_eq!(open(&kek, &token, b"session-aad").unwrap(), b"{\"email\":\"a@b.c\"}");
    }

    #[test]
    fn open_rejects_a_tampered_token() {
        let kek = test_kek();
        let token = seal(&kek, b"payload", b"aad").unwrap();
        let mut bytes = token.into_bytes();
        bytes[0] ^= 0x01;
        let tampered = String::from_utf8(bytes).unwrap();

        assert!(matches!(
            open(&kek, &tampered, b"aad"),
            Err(EncryptionError::DecryptFailed)
        ));
    }

    #[test]
    fn open_rejects_a_wrong_aad() {
        let kek = test_kek();
        let token = seal(&kek, b"payload", b"good-aad").unwrap();

        assert!(open(&kek, &token, b"bad-aad").is_err());
    }

    #[test]
    fn open_rejects_garbage() {
        let kek = test_kek();

        assert!(matches!(
            open(&kek, "not-a-sealed-token", b"aad"),
            Err(EncryptionError::DecryptFailed)
        ));
    }

    #[test]
    fn encrypt_then_decrypt_returns_original_plaintext() {
        let kek = test_kek();
        let plaintext = b"apiVersion: v1\nkind: Config\n";
        let aad = b"cluster-id||1";

        let encrypted = encrypt(&kek, plaintext, aad, CURRENT_KEY_VERSION).unwrap();
        let decrypted = decrypt(&kek, &encrypted, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn distinct_encryptions_produce_distinct_ciphertexts() {
        let kek = test_kek();
        let plaintext = b"same input";
        let aad = b"aad";

        let a = encrypt(&kek, plaintext, aad, CURRENT_KEY_VERSION).unwrap();
        let b = encrypt(&kek, plaintext, aad, CURRENT_KEY_VERSION).unwrap();

        assert_ne!(a.ciphertext, b.ciphertext);
        assert_ne!(a.nonce, b.nonce);
    }

    #[test]
    fn decrypt_with_wrong_aad_fails() {
        let kek = test_kek();
        let encrypted = encrypt(&kek, b"secret", b"good-aad", CURRENT_KEY_VERSION).unwrap();

        assert!(decrypt(&kek, &encrypted, b"bad-aad").is_err());
    }

    #[test]
    fn decrypt_with_wrong_kek_fails() {
        let encrypted = encrypt(&test_kek(), b"secret", b"aad", CURRENT_KEY_VERSION).unwrap();
        let other_kek = Kek::from_bytes([9u8; 32]);

        assert!(decrypt(&other_kek, &encrypted, b"aad").is_err());
    }

    #[test]
    fn decrypt_with_tampered_ciphertext_fails() {
        let kek = test_kek();
        let mut encrypted = encrypt(&kek, b"secret", b"aad", CURRENT_KEY_VERSION).unwrap();
        encrypted.ciphertext[0] ^= 0xff;

        assert!(decrypt(&kek, &encrypted, b"aad").is_err());
    }

    #[test]
    fn kek_from_base64_rejects_wrong_length() {
        assert!(Kek::from_base64("c2hvcnQ=").is_err());
    }

    #[test]
    fn kek_from_base64_accepts_valid_32_byte_key() {
        let raw = [42u8; 32];
        let encoded = base64::engine::general_purpose::STANDARD.encode(raw);
        let kek = Kek::from_base64(&encoded).unwrap();
        assert_eq!(kek.0, raw);
    }

    #[test]
    fn debug_does_not_leak_ciphertext_or_dek() {
        let kek = test_kek();
        let encrypted = encrypt(&kek, b"super secret", b"aad", CURRENT_KEY_VERSION).unwrap();
        let debug = format!("{:?}", encrypted);

        assert!(!debug.contains("super secret"));
        assert!(debug.contains("ciphertext_len"));
        assert!(debug.contains("dek_ciphertext_len"));
        assert!(!debug.contains(&format!("{:?}", encrypted.ciphertext)));
    }

    #[test]
    fn decrypt_with_wrong_nonce_length_fails() {
        let kek = test_kek();
        let mut encrypted = encrypt(&kek, b"secret", b"aad", CURRENT_KEY_VERSION).unwrap();
        encrypted.nonce = vec![0u8; 8];

        assert!(matches!(
            decrypt(&kek, &encrypted, b"aad"),
            Err(EncryptionError::InvalidNonceLength(8))
        ));
    }
}
