use crate::error::{EncryptionError, KittyError};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm,
};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use x25519_dalek::{PublicKey, StaticSecret};

/// Encryptor for kitty remote control password authentication.
///
/// Kitty uses X25519 ECDH for key exchange with AES-256-GCM encryption.
/// The public key format is `1:<base85_encoded_key>` where `1` is the
/// protocol version (currently only one protocol exists).
pub struct Encryptor {
    kitty_public_key: PublicKey,
}

impl Encryptor {
    pub fn new() -> Result<Self, EncryptionError> {
        let kitty_public_key = Self::load_kitty_public_key()?;
        Ok(Self { kitty_public_key })
    }

    pub fn new_with_public_key(public_key: Option<&str>) -> Result<Self, EncryptionError> {
        let kitty_public_key = if let Some(pk) = public_key {
            Self::parse_public_key(pk)?
        } else {
            let key_bytes = Self::read_kitty_public_key()?;
            Self::bytes_to_public_key(&key_bytes)?
        };

        Ok(Self { kitty_public_key })
    }

    fn load_kitty_public_key() -> Result<PublicKey, EncryptionError> {
        let key_bytes = Self::read_kitty_public_key()?;
        Self::bytes_to_public_key(&key_bytes)
    }

    fn parse_public_key(key_str: &str) -> Result<PublicKey, EncryptionError> {
        let key_data = key_str.strip_prefix("1:").ok_or_else(|| {
            EncryptionError::InvalidPublicKey("Missing version prefix".to_string())
        })?;
        let key_bytes = base85::decode(key_data)
            .map_err(|e| EncryptionError::InvalidPublicKey(e.to_string()))?;
        Self::bytes_to_public_key(&key_bytes)
    }

    fn bytes_to_public_key(key_bytes: &[u8]) -> Result<PublicKey, EncryptionError> {
        if key_bytes.len() < 32 {
            return Err(EncryptionError::PublicKeyTooShort {
                expected: 32,
                actual: key_bytes.len(),
            });
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes[..32]);
        Ok(PublicKey::from(key_array))
    }

    /// Read kitty's public key from environment.
    ///
    /// KITTY_PUBLIC_KEY format is `1:<base85_encoded_key>` where:
    /// - `1`: Protocol version (currently only one exists)
    /// - `<base85_encoded_key>`: X25519 public key in Base85 encoding
    ///
    /// This env var is set by kitty when launching subprocesses,
    /// so this method works for processes launched by kitty.
    fn read_kitty_public_key() -> Result<Vec<u8>, EncryptionError> {
        if let Ok(key_str) = std::env::var("KITTY_PUBLIC_KEY") {
            let key_data = key_str.strip_prefix("1:").ok_or_else(|| {
                EncryptionError::InvalidPublicKey("Missing version prefix".to_string())
            })?;
            return base85::decode(key_data)
                .map_err(|e| EncryptionError::InvalidPublicKey(e.to_string()));
        }

        let default_path = format!(
            "{}/.config/kitty/key.pub",
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        );

        let key_path = Path::new(&default_path);
        if !key_path.exists() {
            return Err(EncryptionError::MissingPublicKey);
        }

        let key_bytes =
            fs::read(&key_path).map_err(|e| EncryptionError::InvalidPublicKey(e.to_string()))?;

        Ok(key_bytes)
    }

    pub fn encrypt_command(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, KittyError> {
        let payload_str = serde_json::to_string(&payload)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let payload_bytes = payload_str.as_bytes();

        let secret = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = PublicKey::from(&secret);
        let shared_secret = secret.diffie_hellman(&self.kitty_public_key);

        let mut hasher = Sha256::new();
        hasher.update(shared_secret.as_bytes());
        let encryption_key = hasher.finalize();

        let cipher = Aes256Gcm::new_from_slice(&encryption_key)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, payload_bytes)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let tag = &ciphertext[ciphertext.len() - 16..];
        let encrypted_data = &ciphertext[..ciphertext.len() - 16];

        let result = serde_json::json!({
            "version": "0.43.1",
            "iv": base85::encode(&nonce),
            "tag": base85::encode(tag),
            "pubkey": base85::encode(public_key.as_bytes()),
            "encrypted": base85::encode(encrypted_data),
        });

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_kitty_public_key_missing() {
        // Note: unsafe is required to modify env vars in Rust tests
        unsafe {
            std::env::remove_var("KITTY_PUBLIC_KEY");
        }
        let result = Encryptor::new();
        assert!(matches!(result, Err(EncryptionError::MissingPublicKey)));
    }

    #[test]
    fn test_load_kitty_public_key_invalid() {
        // Note: unsafe is required to modify env vars in Rust tests
        unsafe {
            std::env::set_var("KITTY_PUBLIC_KEY", "invalid base85");
        }
        let result = Encryptor::new();
        assert!(matches!(result, Err(EncryptionError::InvalidPublicKey(_))));
    }

    #[test]
    fn test_load_kitty_public_key_too_short() {
        let short_key = format!("1:{}", base85::encode(&[1u8, 2, 3]));
        // Note: unsafe is required to modify env vars in Rust tests
        unsafe {
            std::env::set_var("KITTY_PUBLIC_KEY", short_key);
        }
        let result = Encryptor::new();
        assert!(matches!(
            result,
            Err(EncryptionError::PublicKeyTooShort { .. })
        ));
    }

    #[test]
    fn test_new_with_public_key() {
        let secret = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = PublicKey::from(&secret);
        let public_key_str = format!("1:{}", base85::encode(public_key.as_bytes()));

        let encryptor = Encryptor::new_with_public_key(Some(&public_key_str));
        assert!(encryptor.is_ok());
    }

    #[test]
    fn test_new_with_public_key_invalid() {
        let encryptor = Encryptor::new_with_public_key(Some("invalid base85"));
        assert!(matches!(
            encryptor,
            Err(EncryptionError::InvalidPublicKey(_))
        ));
    }

    #[test]
    fn test_new_with_public_key_none() {
        let secret = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = PublicKey::from(&secret);
        // Note: unsafe is required to modify env vars in Rust tests
        unsafe {
            std::env::set_var(
                "KITTY_PUBLIC_KEY",
                format!("1:{}", base85::encode(public_key.as_bytes())),
            );
        }

        let encryptor = Encryptor::new_with_public_key(None);
        assert!(encryptor.is_ok());
    }

    #[test]
    fn test_encrypt_command() {
        let secret = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = PublicKey::from(&secret);
        // Note: unsafe is required to modify env vars in Rust tests
        unsafe {
            std::env::set_var(
                "KITTY_PUBLIC_KEY",
                format!("1:{}", base85::encode(public_key.as_bytes())),
            );
        }

        let encryptor = Encryptor::new().unwrap();
        let payload = serde_json::json!({"cmd": "ls", "password": "test", "timestamp": 1234567890});

        let result = encryptor.encrypt_command(payload);
        assert!(result.is_ok());

        let encrypted = result.unwrap();
        assert!(encrypted.is_object());
        let obj = encrypted.as_object().unwrap();
        assert!(obj.contains_key("version"));
        assert!(obj.contains_key("iv"));
        assert!(obj.contains_key("tag"));
        assert!(obj.contains_key("pubkey"));
        assert!(obj.contains_key("encrypted"));
    }
}
