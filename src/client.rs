use crate::encryption::Encryptor;
use crate::error::{ConnectionError, EncryptionError, KittyError};
use crate::protocol::{KittyMessage, KittyResponse};
use std::path::Path;
use std::process::Command;
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

    fn extract_pid_from_socket(socket_path: &str) -> Option<u32> {
        let filename = Path::new(socket_path)
            .file_name()?
            .to_str()?;

        let pid_str = filename.strip_prefix("kitty-")?;
        let pid_str = pid_str.strip_suffix(".sock")?;
        pid_str.parse().ok()
    }

    fn query_public_key_database(pid: u32) -> Result<Option<String>, EncryptionError> {
        let output = Command::new("kitty-pubkey-db")
            .arg("get")
            .arg(pid.to_string())
            .output()
            .map_err(|e| {
                EncryptionError::PublicKeyDatabaseError(format!("Failed to run kitty-pubkey-db: {}", e))
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let pubkey = String::from_utf8(output.stdout)
            .map_err(|e| {
                EncryptionError::PublicKeyDatabaseError(format!("Invalid UTF-8 output: {}", e))
            })?
            .trim()
            .to_string();

        if pubkey.is_empty() {
            Ok(None)
        } else {
            Ok(Some(pubkey))
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

    /// Set kitty's public key explicitly.
    ///
    /// Format: `1:<base85_encoded_key>` where `1` is protocol version.
    ///
    /// When set, this key is used instead of querying KITTY_PUBLIC_KEY
    /// env var or kitty-pubkey-db database.
    ///
    /// Example:
    /// ```no_run
    /// use kitty_rc::Kitty;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let kitty = Kitty::builder()
    ///     .socket_path("/run/user/1000/kitty-12345.sock")
    ///     .password("your-password")
    ///     .public_key("1:z3;{}!NzNzgiXreB&ywA!8y1H8hq^$cMG!OE$QNa")
    ///     .connect()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn public_key(mut self, public_key: impl Into<String>) -> Self {
        self.public_key = Some(public_key.into());
        self
    }

    /// Connect to kitty instance with configured authentication.
    ///
    /// Public key resolution order (when password is set):
    /// 1. Explicit key set via `.public_key()` method
    /// 2. Query kitty-pubkey-db database (extracts PID from socket path)
    /// 3. KITTY_PUBLIC_KEY environment variable (set by kitty when launching subprocesses)
    ///
    /// When no password is set, no encryption is used.
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
            let public_key = if let Some(pk) = self.public_key {
                Some(pk)
            } else if let Some(pid) = Self::extract_pid_from_socket(&socket_path) {
                Self::query_public_key_database(pid).map_err(KittyError::Encryption)?
            } else {
                None
            };

            Some(Encryptor::new_with_public_key(public_key.as_deref())?)
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

    #[test]
    fn test_extract_pid_from_socket_standard() {
        let pid = KittyBuilder::extract_pid_from_socket("/tmp/kitty-12345.sock");
        assert_eq!(pid, Some(12345));
    }

    #[test]
    fn test_extract_pid_from_socket_xdg_runtime_dir() {
        let pid = KittyBuilder::extract_pid_from_socket(
            "/run/user/1000/kitty-67890.sock",
        );
        assert_eq!(pid, Some(67890));
    }

    #[test]
    fn test_extract_pid_from_socket_invalid() {
        let pid = KittyBuilder::extract_pid_from_socket("/tmp/invalid.sock");
        assert_eq!(pid, None);
    }

    #[test]
    fn test_extract_pid_from_socket_no_prefix() {
        let pid = KittyBuilder::extract_pid_from_socket("/tmp/12345.sock");
        assert_eq!(pid, None);
    }

    #[test]
    fn test_extract_pid_from_socket_invalid_pid() {
        let pid = KittyBuilder::extract_pid_from_socket("/tmp/kitty-abc.sock");
        assert_eq!(pid, None);
    }

    #[tokio::test]
    async fn test_builder_missing_socket() {
        let builder = KittyBuilder::new();
        let result = builder.connect().await;

        assert!(result.is_err());
    }
}
