use crate::error::ProtocolError;
use crate::protocol::{KittyMessage, KittyResponse};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Failed to connect to socket: {0}")]
    ConnectionError(#[from] std::io::Error),

    #[error("Connection timeout after {0:?}")]
    TimeoutError(Duration),

    #[error("Failed to send message: {0}")]
    SendError(String),

    #[error("Failed to receive response: {0}")]
    ReceiveError(String),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
}

pub struct KittyClient {
    socket_path: String,
    stream: Option<UnixStream>,
    timeout: Duration,
}

impl KittyClient {
    pub async fn connect<P: AsRef<Path>>(path: P) -> Result<Self, TransportError> {
        Self::connect_with_timeout(path, Duration::from_secs(10)).await
    }

    pub async fn connect_with_timeout<P: AsRef<Path>>(
        path: P,
        timeout_duration: Duration,
    ) -> Result<Self, TransportError> {
        let stream = timeout(timeout_duration, UnixStream::connect(&path))
            .await
            .map_err(|_| TransportError::TimeoutError(timeout_duration))??;

        Ok(Self {
            socket_path: path.as_ref().to_string_lossy().to_string(),
            stream: Some(stream),
            timeout: timeout_duration,
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn ensure_connected(&mut self) -> Result<(), TransportError> {
        if self.stream.is_none() {
            let stream = timeout(self.timeout, UnixStream::connect(&self.socket_path))
                .await
                .map_err(|_| TransportError::TimeoutError(self.timeout))??;
            self.stream = Some(stream);
        }
        Ok(())
    }

    pub async fn send(&mut self, message: &KittyMessage) -> Result<(), TransportError> {
        self.ensure_connected().await?;

        let data = message.encode()?;
        let stream = self.stream.as_mut().ok_or(TransportError::ConnectionClosed)?;

        timeout(self.timeout, stream.write_all(&data))
            .await
            .map_err(|_| TransportError::TimeoutError(self.timeout))??;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<KittyResponse, TransportError> {
        let stream = self.stream.as_mut().ok_or(TransportError::ConnectionClosed)?;

        let mut buffer = vec![0u8; 8192];
        let n = timeout(self.timeout, stream.read(&mut buffer))
            .await
            .map_err(|_| TransportError::TimeoutError(self.timeout))??;

        if n == 0 {
            return Err(TransportError::ConnectionClosed);
        }

        buffer.truncate(n);
        Ok(KittyResponse::decode(&buffer)?)
    }

    pub async fn execute(&mut self, message: &KittyMessage) -> Result<KittyResponse, TransportError> {
        self.send(message).await?;
        self.receive().await
    }

    pub async fn reconnect(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }

        let new_stream = timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| TransportError::TimeoutError(self.timeout))??;

        self.stream = Some(new_stream);
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await.ok();
        }
        Ok(())
    }
}

impl Drop for KittyClient {
    fn drop(&mut self) {
        if let Some(_stream) = self.stream.take() {
            // The stream will be closed when dropped
        }
    }
}

pub struct ConnectionPool {
    socket_path: String,
    timeout: Duration,
    max_size: usize,
    connections: Vec<KittyClient>,
}

impl ConnectionPool {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            socket_path: path.as_ref().to_string_lossy().to_string(),
            timeout: Duration::from_secs(10),
            max_size: 10,
            connections: Vec::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub async fn acquire(&mut self) -> Result<KittyClient, TransportError> {
        if let Some(mut client) = self.connections.pop() {
            client.ensure_connected().await?;
            Ok(client)
        } else {
            KittyClient::connect_with_timeout(&self.socket_path, self.timeout).await
        }
    }

    pub fn release(&mut self, client: KittyClient) {
        if self.connections.len() < self.max_size {
            self.connections.push(client);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = KittyClient::connect("/nonexistent/socket").await;
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_client_timeout() {
        let client = KittyClient::connect_with_timeout("/nonexistent/socket", Duration::from_millis(100)).await;
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ConnectionPool::new("/tmp/test.sock")
            .with_timeout(Duration::from_secs(5))
            .with_max_size(5);

        assert_eq!(pool.max_size, 5);
    }

    #[test]
    fn test_error_display() {
        let err = TransportError::ConnectionClosed;
        assert_eq!(err.to_string(), "Connection closed unexpectedly");
    }
}
