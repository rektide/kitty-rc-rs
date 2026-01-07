use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidMessageFormat(String),

    #[error("Failed to encode/decode JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid escape sequence in message")]
    InvalidEscapeSequence,

    #[error("Response envelope parsing failed: {0}")]
    EnvelopeParseError(String),
}
