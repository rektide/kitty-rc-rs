use thiserror::Error;

/// Errors from decoding or classifying kitty remote-control wire data.
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid envelope: {0}")]
    InvalidEnvelope(String),
}
