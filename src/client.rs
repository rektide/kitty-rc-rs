use crate::error::{ConnectionError, KittyError};
use crate::protocol::{KittyMessage, KittyResponse};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

pub struct Kitty {
    stream: UnixStream,
    timeout: Duration,
    socket_path: String,
}

pub struct KittyBuilder {
    socket_path: Option<String>,
    timeout: Duration,
}

impl KittyBuilder {
    pub fn new() -> Self {
        Self {
            socket_path: None,
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

    pub async fn connect(self) -> Result<Kitty, KittyError> {
        let socket_path = self
            .socket_path
            .ok_or_else(|| KittyError::Connection(ConnectionError::SocketNotFound(
                "No socket path provided".to_string(),
            )))?;

        let stream = timeout(self.timeout, UnixStream::connect(&socket_path))
            .await
            .map_err(|_| ConnectionError::TimeoutError(self.timeout))?
            .map_err(|e| ConnectionError::ConnectionFailed(socket_path.clone(), e))?;

        Ok(Kitty {
            stream,
            timeout: self.timeout,
            socket_path,
        })
    }
}

impl Kitty {
    pub fn builder() -> KittyBuilder {
        KittyBuilder::new()
    }
    async fn send(&mut self, message: &KittyMessage) -> Result<(), KittyError> {
        let data = message.encode()?;

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
                self.send(&chunk).await?;
            }
        } else {
            self.send(message).await?;
        }
        Ok(())
    }

    pub async fn execute_all(&mut self, message: &KittyMessage) -> Result<KittyResponse, KittyError> {
        self.send_all(message).await?;
        self.receive().await
    }

    pub async fn send_command<T: Into<KittyMessage>>(&mut self, command: T) -> Result<(), KittyError> {
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

    #[tokio::test]
    async fn test_builder_missing_socket() {
        let builder = KittyBuilder::new();
        let result = builder.connect().await;
        assert!(result.is_err());
    }
}
