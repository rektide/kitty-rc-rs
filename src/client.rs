use crate::encryption::Encryptor;
use crate::error::{ConnectionError, KittyError};
use crate::protocol::{KittyMessage, KittyResponse};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

pub struct Kitty {
    stream: UnixStream,
    timeout: Duration,
    socket_path: String,
    password: Option<String>,
    encryptor: Option<Encryptor>,
}

pub struct KittyBuilder {
    socket_path: Option<String>,
    password: Option<String>,
    public_key: Option<String>,
    timeout: Duration,
}

impl KittyBuilder {
    pub fn new() -> Self {
        Self {
            socket_path: None,
            password: None,
            public_key: None,
            timeout: Duration::from_secs(10),
        }
    }

    pub fn socket_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.socket_path = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn public_key(mut self, public_key: impl Into<String>) -> Self {
        self.public_key = Some(public_key.into());
        self
    }

    pub async fn connect(self) -> Result<Kitty, KittyError> {
        let socket_path = self.socket_path.ok_or_else(|| {
            KittyError::Connection(ConnectionError::SocketNotFound(
                "No socket path provided".to_string(),
            ))
        })?;

        let stream = timeout(self.timeout, UnixStream::connect(&socket_path))
            .await
            .map_err(|_| ConnectionError::TimeoutError(self.timeout))?
            .map_err(|e| ConnectionError::ConnectionFailed(socket_path.clone(), e))?;

        let encryptor = if self.password.is_some() {
            Some(Encryptor::new_with_public_key(self.public_key.as_deref())?)
        } else {
            None
        };

        Ok(Kitty {
            stream,
            timeout: self.timeout,
            socket_path,
            password: self.password,
            encryptor,
        })
    }
}

impl Kitty {
    pub fn builder() -> KittyBuilder {
        KittyBuilder::new()
    }

    fn encrypt_command(&self, mut message: KittyMessage) -> Result<KittyMessage, KittyError> {
        let Some(encryptor) = &self.encryptor else {
            return Ok(message);
        };

        let Some(password) = &self.password else {
            return Ok(message);
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| {
                KittyError::Encryption(crate::error::EncryptionError::EncryptionFailed(
                    "Failed to get timestamp".to_string(),
                ))
            })?
            .as_nanos();

        if let Some(payload) = &mut message.payload {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("password".to_string(), serde_json::json!(password));
                obj.insert("timestamp".to_string(), serde_json::json!(timestamp));
            }
        } else {
            let mut obj = serde_json::Map::new();
            obj.insert("password".to_string(), serde_json::json!(password));
            obj.insert("timestamp".to_string(), serde_json::json!(timestamp));
            message.payload = Some(serde_json::Value::Object(obj));
        }

        let encrypted_payload = encryptor.encrypt_command(message.payload.unwrap())?;
        message.payload = Some(encrypted_payload);

        Ok(message)
    }

    async fn send(&mut self, message: &KittyMessage) -> Result<(), KittyError> {
        let encrypted_msg = self.encrypt_command(message.clone())?;
        let data = encrypted_msg.encode()?;

        timeout(self.timeout, self.stream.write_all(&data))
            .await
            .map_err(|_| ConnectionError::TimeoutError(self.timeout))??;

        Ok(())
    }

    async fn receive(&mut self) -> Result<KittyResponse, KittyError> {
        const SUFFIX: &[u8] = b"\x1b\\";

        let mut buffer = Vec::new();

        loop {
            let mut chunk = vec![0u8; 8192];
            let n = timeout(self.timeout, self.stream.read(&mut chunk))
                .await
                .map_err(|_| ConnectionError::TimeoutError(self.timeout))??;

            if n == 0 {
                break;
            }

            buffer.extend_from_slice(&chunk[..n]);

            if buffer.ends_with(SUFFIX) {
                break;
            }
        }

        if buffer.is_empty() {
            return Err(KittyError::Connection(ConnectionError::ConnectionClosed));
        }

        Ok(KittyResponse::decode(&buffer)?)
    }

    pub async fn execute(&mut self, message: &KittyMessage) -> Result<KittyResponse, KittyError> {
        self.send(message).await?;
        self.receive().await
    }

    pub async fn send_all(&mut self, message: &KittyMessage) -> Result<(), KittyError> {
        if message.needs_streaming() {
            for chunk in message.clone().into_chunks() {
                let encrypted_chunk = self.encrypt_command(chunk)?;
                self.send(&encrypted_chunk).await?;
            }
        } else {
            let encrypted_msg = self.encrypt_command(message.clone())?;
            self.send(&encrypted_msg).await?;
        }

        Ok(())
    }

    pub async fn execute_all(
        &mut self,
        message: &KittyMessage,
    ) -> Result<KittyResponse, KittyError> {
        self.send_all(message).await?;
        self.receive().await
    }

    pub async fn send_command<T: Into<KittyMessage>>(
        &mut self,
        command: T,
    ) -> Result<(), KittyError> {
        self.send_all(&command.into()).await
    }

    pub async fn reconnect(&mut self) -> Result<(), KittyError> {
        let _ = self.stream.shutdown().await;

        let new_stream = timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| ConnectionError::TimeoutError(self.timeout))?
            .map_err(|e| ConnectionError::ConnectionFailed(self.socket_path.clone(), e))?;

        self.stream = new_stream;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), KittyError> {
        self.stream.shutdown().await.ok();
        Ok(())
    }
}

impl Drop for Kitty {
    fn drop(&mut self) {
        let _ = self.stream.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = KittyBuilder::new()
            .socket_path("/tmp/test.sock")
            .timeout(Duration::from_secs(5));

        assert_eq!(builder.socket_path, Some("/tmp/test.sock".to_string()));
        assert_eq!(builder.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_builder_with_password() {
        let builder = KittyBuilder::new().password("test-password");

        assert_eq!(builder.password, Some("test-password".to_string()));
    }

    #[test]
    fn test_builder_with_public_key() {
        let builder = KittyBuilder::new().public_key("1:abc123");

        assert_eq!(builder.public_key, Some("1:abc123".to_string()));
    }

    #[tokio::test]
    async fn test_builder_missing_socket() {
        let builder = KittyBuilder::new();
        let result = builder.connect().await;

        assert!(result.is_err());
    }
}
