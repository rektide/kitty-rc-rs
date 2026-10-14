//! One-envelope request protection for kitty remote control.
//!
//! This crate is the **only** outbound protection boundary. A plaintext
//! request ([`PlainRequest`] serialized to JSON) becomes a
//! [`ProtectedEnvelope`] exactly once, here; transport and exchange code
//! receives framed bytes and can never encrypt a second time.
//!
//! The algorithm is kitty's, as implemented by `kitty/remote_control.py`
//! (`CommandEncrypter`) and `kitty/crypto.c`, and documented in
//! `docs/rc_protocol.rst`:
//!
//! 1. generate an ephemeral X25519 key pair;
//! 2. derive the symmetric key as `SHA-256(X25519(ephemeral secret, server public key))`;
//! 3. add `password` and `timestamp` (nanoseconds since the Unix epoch) to
//!    the **top level** of the plaintext command object;
//! 4. seal the serialized command with AES-256-GCM (random 96-bit IV,
//!    128-bit tag);
//! 5. emit the wire envelope `{version, iv, tag, pubkey, encrypted}` with all
//!    binary fields RFC 1924 base85-encoded (Python's `base64.b85*`).
//!
//! Everything is explicit: no environment variables, no files, no subprocess
//! key lookup, and no clock reads — the caller supplies the server key,
//! password, timestamp, and randomness. Servers reject timestamps more than
//! five minutes away; choosing the timestamp is the caller's policy.
//!
//! Keys are accepted only in kitty's published format: `1:<base85>`, where
//! `1` is the encryption protocol version and the decoded key is exactly 32
//! bytes. Shorter or longer keys are errors, not silently truncated.

use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use kitty_rc_schema::{PlainRequest, ProtectedEnvelope};
use rand_core::CryptoRngCore;
use sha2::{Digest, Sha256};
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};

/// IV length kitty generates for AES-256-GCM: 96 bits.
pub const IV_LEN: usize = 12;
/// AEAD tag length: 128 bits.
pub const TAG_LEN: usize = 16;
/// X25519 public key length.
pub const KEY_LEN: usize = 32;
/// The only encryption protocol version kitty currently defines.
pub const ENCRYPTION_PROTOCOL_VERSION: &str = "1";

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("unsupported encryption protocol version {version:?}; only {SUPPORTED_VERSIONS} exist", SUPPORTED_VERSIONS = ENCRYPTION_PROTOCOL_VERSION)]
    UnsupportedProtocol { version: String },

    #[error("invalid public key: {reason}")]
    InvalidPublicKey { reason: String },

    #[error("plaintext command must be a JSON object to carry password and timestamp")]
    PayloadNotAnObject,

    #[error("decryption failed: field {field} did not decode ({reason})")]
    UndecodableField { field: &'static str, reason: String },

    #[error("authenticated decryption failed (wrong key or tampered envelope)")]
    AuthenticationFailed,

    #[error("non-contributory X25519 public key")]
    NonContributoryKey,

    #[error("decrypted plaintext was not a JSON object")]
    PlaintextNotAnObject,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// A parsed server X25519 public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPublicKey(PublicKey);

impl ServerPublicKey {
    /// Parse kitty's published key format `1:<base85>` (as carried in
    /// `KITTY_PUBLIC_KEY` and key stores). The protocol prefix must be `1`
    /// and the decoded key must be exactly [`KEY_LEN`] bytes.
    pub fn parse(published: &str) -> Result<Self, CryptoError> {
        let (version, encoded) =
            published
                .split_once(':')
                .ok_or_else(|| CryptoError::InvalidPublicKey {
                    reason: "missing '<version>:' prefix".to_string(),
                })?;
        if version != ENCRYPTION_PROTOCOL_VERSION {
            return Err(CryptoError::UnsupportedProtocol {
                version: version.to_string(),
            });
        }
        let bytes = base85::decode(encoded).map_err(|e| CryptoError::InvalidPublicKey {
            reason: format!("not RFC 1924 base85: {e}"),
        })?;
        let key: [u8; KEY_LEN] =
            bytes
                .try_into()
                .map_err(|b: Vec<u8>| CryptoError::InvalidPublicKey {
                    reason: format!(
                        "decoded key must be exactly {KEY_LEN} bytes, got {}",
                        b.len()
                    ),
                })?;
        Ok(Self(PublicKey::from(key)))
    }

    /// The raw 32-byte key (for evidence and tests).
    pub fn as_bytes(&self) -> [u8; KEY_LEN] {
        *self.0.as_bytes()
    }
}

/// Protect one serialized plaintext request.
///
/// `plain` must be the JSON object form of a [`PlainRequest`] (or another
/// command object); `password` and `timestamp` are injected at its top level
/// before sealing, matching kitty's `CommandEncrypter`. `timestamp` is
/// nanoseconds since the Unix epoch. Fresh ephemeral key material and a fresh
/// IV are drawn from `rng` for every call.
pub fn encrypt_raw_request(
    plain: serde_json::Value,
    password: &str,
    timestamp_ns: u64,
    server_key: &ServerPublicKey,
    rng: &mut impl CryptoRngCore,
) -> Result<ProtectedEnvelope, CryptoError> {
    let mut command = plain;
    let object = command
        .as_object_mut()
        .ok_or(CryptoError::PayloadNotAnObject)?;
    object.insert(
        "password".to_string(),
        serde_json::Value::String(password.to_string()),
    );
    object.insert(
        "timestamp".to_string(),
        serde_json::Value::from(timestamp_ns),
    );

    let plaintext = serde_json::to_vec(&command)?;

    let ephemeral = StaticSecret::random_from_rng(&mut *rng);
    let sender_public = PublicKey::from(&ephemeral);
    let shared = ephemeral.diffie_hellman(&server_key.0);
    if !shared.was_contributory() {
        return Err(CryptoError::NonContributoryKey);
    }
    let encryption_key = Sha256::digest(shared.as_bytes());

    let cipher =
        Aes256Gcm::new_from_slice(&encryption_key).expect("SHA-256 digest is a valid AES-256 key");
    let mut iv_bytes = [0u8; IV_LEN];
    rng.fill_bytes(&mut iv_bytes);
    let nonce = GenericArray::from_slice(&iv_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_slice())
        .map_err(|_| CryptoError::AuthenticationFailed)?;
    let (encrypted, tag) = ciphertext.split_at(ciphertext.len() - TAG_LEN);

    let request_version = command
        .get("version")
        .cloned()
        .unwrap_or_else(|| serde_json::to_value(kitty_rc_schema::SUPPORTED).expect("serializable"));

    Ok(ProtectedEnvelope {
        version: serde_json::from_value(request_version)?,
        iv: base85::encode(nonce.as_slice()),
        tag: base85::encode(tag),
        pubkey: base85::encode(sender_public.as_bytes()),
        encrypted: base85::encode(encrypted),
        enc_proto: None,
    })
}

/// Typed protection entrance: a protected envelope cannot be passed here.
pub fn encrypt_request(
    plain: &PlainRequest,
    password: &str,
    timestamp_ns: u64,
    server_key: &ServerPublicKey,
    rng: &mut impl CryptoRngCore,
) -> Result<ProtectedEnvelope, CryptoError> {
    encrypt_raw_request(
        serde_json::to_value(plain)?,
        password,
        timestamp_ns,
        server_key,
        rng,
    )
}

/// Decrypt a protected envelope using the **recipient's** private key,
/// mirroring what a kitty server does in `parse_cmd`.
///
/// This exists for conformance fixtures and verification: a test (or audit
/// tool) holding the server-side secret can confirm what a client would
/// actually send, independently of the encryption path. It is not part of any
/// normal client flow; responses from kitty are not encrypted.
pub fn decrypt_envelope(
    envelope: &ProtectedEnvelope,
    recipient_secret: &StaticSecret,
) -> Result<serde_json::Value, CryptoError> {
    let decode_field = |field: &'static str, encoded: &str| -> Result<Vec<u8>, CryptoError> {
        base85::decode(encoded).map_err(|e| CryptoError::UndecodableField {
            field,
            reason: e.to_string(),
        })
    };

    let sender_public = decode_field("pubkey", &envelope.pubkey)?;
    let sender_public: [u8; KEY_LEN] =
        sender_public
            .try_into()
            .map_err(|b: Vec<u8>| CryptoError::UndecodableField {
                field: "pubkey",
                reason: format!("must decode to exactly {KEY_LEN} bytes, got {}", b.len()),
            })?;
    let iv = decode_field("iv", &envelope.iv)?;
    let tag = decode_field("tag", &envelope.tag)?;
    let encrypted = decode_field("encrypted", &envelope.encrypted)?;

    let shared = recipient_secret.diffie_hellman(&PublicKey::from(sender_public));
    if !shared.was_contributory() {
        return Err(CryptoError::NonContributoryKey);
    }
    let encryption_key = Sha256::digest(shared.as_bytes());
    let cipher =
        Aes256Gcm::new_from_slice(&encryption_key).expect("SHA-256 digest is a valid AES-256 key");

    let mut ciphertext = encrypted;
    ciphertext.extend_from_slice(&tag);
    let iv_bytes: [u8; IV_LEN] =
        iv.try_into()
            .map_err(|b: Vec<u8>| CryptoError::UndecodableField {
                field: "iv",
                reason: format!("must decode to exactly {IV_LEN} bytes, got {}", b.len()),
            })?;

    let plaintext = cipher
        .decrypt(GenericArray::from_slice(&iv_bytes), ciphertext.as_slice())
        .map_err(|_| CryptoError::AuthenticationFailed)?;

    let value: serde_json::Value = serde_json::from_slice(&plaintext)?;
    if !value.is_object() {
        return Err(CryptoError::PlaintextNotAnObject);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kitty_rc_schema::PlainRequest;
    use kitty_rc_schema::SUPPORTED;
    use rand_core::OsRng;

    fn server_pair() -> (StaticSecret, ServerPublicKey) {
        let secret = StaticSecret::random_from_rng(OsRng);
        let key = ServerPublicKey(PublicKey::from(&secret));
        (secret, key)
    }

    fn published(key: &ServerPublicKey) -> String {
        format!(
            "{ENCRYPTION_PROTOCOL_VERSION}:{}",
            base85::encode(key.as_bytes().as_slice())
        )
    }

    fn sample_request() -> PlainRequest {
        let mut request = PlainRequest::new("ls");
        request.payload = Some(serde_json::json!({"match": "id:1"}));
        request
    }

    #[test]
    fn parses_published_key_format() {
        let (_, key) = server_pair();
        let parsed = ServerPublicKey::parse(&published(&key)).unwrap();
        assert_eq!(parsed, key);
        assert_eq!(parsed.as_bytes().len(), KEY_LEN);
    }

    #[test]
    fn rejects_wrong_protocol_prefix() {
        let error = ServerPublicKey::parse("2:abc").unwrap_err();
        assert!(matches!(error, CryptoError::UnsupportedProtocol { .. }));
    }

    #[test]
    fn rejects_missing_prefix() {
        let error = ServerPublicKey::parse("AAAA").unwrap_err();
        assert!(matches!(error, CryptoError::InvalidPublicKey { .. }));
    }

    #[test]
    fn rejects_non_base85() {
        let error = ServerPublicKey::parse("1:not base85 !!").unwrap_err();
        assert!(matches!(error, CryptoError::InvalidPublicKey { .. }));
    }

    /// Exact-length rule: the legacy implementation silently truncated longer
    /// keys and accepted shorter ones; both are errors now.
    #[test]
    fn rejects_inexact_key_lengths() {
        for length in [0, 1, 31, 33, 64] {
            let published = format!("1:{}", base85::encode(&vec![0u8; length]));
            let error = ServerPublicKey::parse(&published).unwrap_err();
            assert!(
                matches!(error, CryptoError::InvalidPublicKey { .. }),
                "length {length} should be rejected"
            );
        }
    }

    /// Independent decrypt oracle: seal as a client, open as the server.
    #[test]
    fn encrypt_then_server_side_decrypt_round_trip() {
        let (server_secret, server_key) = server_pair();
        let request = sample_request();
        let envelope = encrypt_request(
            &request,
            "pw",
            1_700_000_000_000_000_000,
            &server_key,
            &mut OsRng,
        )
        .unwrap();

        let decrypted = decrypt_envelope(&envelope, &server_secret).unwrap();

        // Password and timestamp live at the top level of the command object,
        // next to the original fields — kitty's parse_cmd contract.
        assert_eq!(decrypted["password"], "pw");
        assert_eq!(decrypted["timestamp"], 1_700_000_000_000_000_000u64);
        assert_eq!(decrypted["cmd"], "ls");
        assert_eq!(
            decrypted["version"],
            serde_json::to_value(SUPPORTED).unwrap()
        );
        assert_eq!(decrypted["payload"]["match"], "id:1");
    }

    #[test]
    fn envelope_field_lengths_and_version() {
        let (server_secret, server_key) = server_pair();
        let envelope =
            encrypt_request(&sample_request(), "pw", 42, &server_key, &mut OsRng).unwrap();

        assert_eq!(envelope.version, SUPPORTED);
        assert_eq!(base85::decode(&envelope.iv).unwrap().len(), IV_LEN);
        assert_eq!(base85::decode(&envelope.tag).unwrap().len(), TAG_LEN);
        assert_eq!(base85::decode(&envelope.pubkey).unwrap().len(), KEY_LEN);
        assert!(!base85::decode(&envelope.encrypted).unwrap().is_empty());
        assert_eq!(envelope.enc_proto, None);
        // The envelope must not carry the password or plaintext command.
        let encoded = serde_json::to_string(&envelope).unwrap();
        assert!(!encoded.contains("pw"), "{encoded}");
        assert!(!encoded.contains("\"cmd\""), "{encoded}");

        // The sender pubkey in the envelope is the ephemeral key, not the
        // server's, and the server can still open it (checked above).
        assert_ne!(
            base85::decode(&envelope.pubkey).unwrap(),
            *PublicKey::from(&server_secret).as_bytes()
        );
    }

    /// Every protection draws fresh ephemeral key material and a fresh IV.
    #[test]
    fn fresh_material_per_call() {
        let (_, server_key) = server_pair();
        let value = sample_request();
        let first = encrypt_request(&value, "pw", 1, &server_key, &mut OsRng).unwrap();
        let second = encrypt_request(&value, "pw", 1, &server_key, &mut OsRng).unwrap();
        assert_ne!(first.iv, second.iv);
        assert_ne!(first.pubkey, second.pubkey);
        assert_ne!(first.encrypted, second.encrypted);
    }

    #[test]
    fn tampered_ciphertext_fails_authentication() {
        let (server_secret, server_key) = server_pair();
        let mut envelope =
            encrypt_request(&sample_request(), "pw", 1, &server_key, &mut OsRng).unwrap();

        let mut encrypted = base85::decode(&envelope.encrypted).unwrap();
        encrypted[0] ^= 0xff;
        envelope.encrypted = base85::encode(&encrypted);

        let error = decrypt_envelope(&envelope, &server_secret).unwrap_err();
        assert!(matches!(error, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn wrong_recipient_fails_authentication() {
        let (_, server_key) = server_pair();
        let envelope =
            encrypt_request(&sample_request(), "pw", 1, &server_key, &mut OsRng).unwrap();

        let other_party = StaticSecret::random_from_rng(OsRng);
        assert!(matches!(
            decrypt_envelope(&envelope, &other_party),
            Err(CryptoError::AuthenticationFailed)
        ));
    }

    #[test]
    fn non_object_payload_is_rejected_before_encryption() {
        let (_, server_key) = server_pair();
        let error = encrypt_raw_request(
            serde_json::json!([1, 2, 3]),
            "pw",
            1,
            &server_key,
            &mut OsRng,
        )
        .unwrap_err();
        assert!(matches!(error, CryptoError::PayloadNotAnObject));
    }

    #[test]
    fn rejects_non_contributory_recipient() {
        let key = ServerPublicKey::parse(&format!("1:{}", base85::encode(&[0; 32]))).unwrap();
        assert!(matches!(
            encrypt_request(&sample_request(), "pw", 1, &key, &mut OsRng),
            Err(CryptoError::NonContributoryKey)
        ));
    }
}
