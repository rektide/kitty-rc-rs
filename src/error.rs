use thiserror::Error;

/// Errors related to protocol message framing, encoding, and decoding
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidMessageFormat(String),

    #[error("Failed to encode/decode JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Missing required field '{0}' in message")]
    MissingField(String),

    #[error("Invalid escape sequence in message")]
    InvalidEscapeSequence,

    #[error("Response envelope parsing failed: {0}")]
    EnvelopeParseError(String),

    #[error("Message payload validation failed: {0}")]
    PayloadValidationError(String),

    #[error("Unsupported protocol version: {0:?}")]
    UnsupportedVersion(Vec<u32>),
}

/// Errors related to command construction, validation, and execution
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command name: '{0}' is not a recognized kitty command")]
    InvalidCommand(String),

    #[error("Missing required parameter '{0}' for command '{1}'")]
    MissingParameter(String, String),

    #[error("Invalid parameter value for '{0}': {1}")]
    InvalidParameter(String, String),

    #[error("Command validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid window match specification: {0}")]
    InvalidWindowMatch(String),

    #[error("Invalid tab match specification: {0}")]
    InvalidTabMatch(String),

    #[error("Invalid layout specification: {0}")]
    InvalidLayout(String),

    #[error("Kitty returned error for command '{0}': {1}")]
    KittyError(String, String),

    #[error("Command execution failed with status: {0}")]
    ExecutionFailed(String),

    #[error("Async command '{0}' was cancelled")]
    AsyncCancelled(String),
}

/// Errors related to encryption and decryption
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption not yet implemented")]
    NotImplemented,

    #[error("KITTY_PUBLIC_KEY environment variable not set")]
    MissingPublicKey,

    #[error("Failed to decode Base85 public key: {0}")]
    InvalidPublicKey(String),

    #[error("Public key too short: expected {expected} bytes, got {actual}")]
    PublicKeyTooShort { expected: usize, actual: usize },

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid public key format")]
    InvalidPublicKeyFormat,
}

/// Errors related to connection, transport, and I/O
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to connect to socket '{0}': {1}")]
    ConnectionFailed(String, #[source] std::io::Error),

    #[error("Connection timeout after {0:?}")]
    TimeoutError(std::time::Duration),

    #[error("Failed to send message: {0}")]
    SendError(String),

    #[error("Failed to receive response: {0}")]
    ReceiveError(String),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    #[error("Socket path '{0}' does not exist")]
    SocketNotFound(String),

    #[error("Permission denied for socket '{0}'")]
    PermissionDenied(String),

    #[error("Maximum retry attempts ({0}) exceeded")]
    MaxRetriesExceeded(usize),
}

/// Top-level error type for the kitty-rc-proto library
///
/// This enum encompasses all possible error types that can occur when
/// interacting with the kitty terminal emulator via the remote control protocol.
#[derive(Error, Debug)]
pub enum KittyError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Command error: {0}")]
    Command(#[from] CommandError),

    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<std::io::Error> for ConnectionError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => ConnectionError::SocketNotFound("unknown".to_string()),
            std::io::ErrorKind::PermissionDenied => {
                ConnectionError::PermissionDenied("unknown".to_string())
            }
            std::io::ErrorKind::ConnectionRefused => {
                ConnectionError::ConnectionFailed("connection refused".to_string(), err)
            }
            _ => ConnectionError::ConnectionFailed("unknown".to_string(), err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_error_display() {
        let err = ProtocolError::InvalidEscapeSequence;
        assert!(err.to_string().contains("escape sequence"));
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::InvalidCommand("fake-cmd".to_string());
        assert!(err.to_string().contains("fake-cmd"));
    }

    #[test]
    fn test_connection_error_display() {
        let err = ConnectionError::ConnectionClosed;
        assert_eq!(err.to_string(), "Connection closed unexpectedly");
    }

    #[test]
    fn test_error_conversion_chain() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let proto_err = ProtocolError::from(json_err);
        let kitty_err: KittyError = proto_err.into();

        assert!(kitty_err.to_string().contains("JSON"));
    }

    #[test]
    fn test_encryption_error_display() {
        let err = EncryptionError::NotImplemented;
        assert!(err.to_string().contains("not yet implemented"));
    }

    #[test]
    fn test_top_level_error() {
        let cmd_err = CommandError::ValidationError("test".to_string());
        let kitty_err: KittyError = cmd_err.into();

        assert!(kitty_err.to_string().contains("validation"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let conn_err = ConnectionError::from(io_err);

        match conn_err {
            ConnectionError::SocketNotFound(_) => assert!(true),
            _ => panic!("Expected SocketNotFound error"),
        }
    }

    #[test]
    fn test_missing_field_error() {
        let err = ProtocolError::MissingField("cmd".to_string());
        assert!(err.to_string().contains("cmd"));
    }

    #[test]
    fn test_parameter_validation_error() {
        let err = CommandError::MissingParameter("match".to_string(), "ls".to_string());
        let msg = err.to_string();
        assert!(msg.contains("match") && msg.contains("ls"));
    }
}
