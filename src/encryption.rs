use crate::error::{EncryptionError, KittyError};
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, AeadCore, KeyInit},
};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::env;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct Encryptor {
    kitty_public_key: PublicKey,
}

impl Encryptor {
    pub fn new() -> Result<Self, EncryptionError> {
        let kitty_public_key = Self::load_kitty_public_key()?;
        Ok(Self { kitty_public_key })
    }

    fn load_kitty_public_key() -> Result<PublicKey, EncryptionError> {
        let key_str =
            env::var("KITTY_PUBLIC_KEY").map_err(|_| EncryptionError::MissingPublicKey)?;

        let key_bytes = base85::decode(&key_str)
            .map_err(|e| EncryptionError::InvalidPublicKey(e.to_string()))?;

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
            "version": "0.26.0",
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
        unsafe {
            env::remove_var("KITTY_PUBLIC_KEY");
        }
        let result = Encryptor::new();
        assert!(matches!(result, Err(EncryptionError::MissingPublicKey)));
    }

    #[test]
    fn test_load_kitty_public_key_invalid() {
        unsafe {
            env::set_var("KITTY_PUBLIC_KEY", "invalid base85");
        }
        let result = Encryptor::new();
        assert!(matches!(result, Err(EncryptionError::InvalidPublicKey(_))));
    }

    #[test]
    fn test_load_kitty_public_key_too_short() {
        let short_key = base85::encode(&[1u8, 2, 3]);
        unsafe {
            env::set_var("KITTY_PUBLIC_KEY", short_key);
        }
        let result = Encryptor::new();
        assert!(matches!(
            result,
            Err(EncryptionError::PublicKeyTooShort { .. })
        ));
    }

    #[test]
    fn test_encrypt_command() {
        let secret = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = PublicKey::from(&secret);
        unsafe {
            env::set_var("KITTY_PUBLIC_KEY", base85::encode(public_key.as_bytes()));
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
