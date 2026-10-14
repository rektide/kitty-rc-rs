use crate::error::SchemaError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A decoded server reply.
///
/// Terminal replies carry `ok` plus optional `data` (a command success) or
/// `error` (a command failure). A streaming handshake acknowledgment carries
/// `ok: true` and `stream: true` and nothing else. Additive unknown fields are
/// tolerated where upstream's responses can grow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// What a [`ResponseEnvelope`] means for the exchange that received it.
///
/// A failure here is the server's own negative answer for the command; it is
/// evidence about the remote outcome, not a transport error and not proof of
/// zero side effects.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseKind {
    /// Acknowledgment of a first nonempty streaming chunk (`ok` + `stream`).
    /// Intermediate input acknowledgment, never the command's final result.
    StreamAcknowledged,
    /// Terminal success. Missing required command data is a typed-output
    /// decoding concern of the command, not of this classification.
    Success { data: Option<Value> },
    /// Terminal failure with the server's error string when present.
    Failure { error: Option<String> },
}

impl ResponseEnvelope {
    /// Ownership-taking classification: large response data is not cloned.
    pub fn into_kind(self) -> Result<ResponseKind, SchemaError> {
        if self.stream == Some(true) {
            if !self.ok || self.data.is_some() || self.error.is_some() {
                return Err(SchemaError::InvalidEnvelope(
                    "stream acknowledgment carries extra terminal fields".into(),
                ));
            }
            return Ok(ResponseKind::StreamAcknowledged);
        }
        if self.ok {
            Ok(ResponseKind::Success { data: self.data })
        } else {
            Ok(ResponseKind::Failure { error: self.error })
        }
    }
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, SchemaError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Classify the reply, rejecting shapes the inspected server never sends.
    pub fn classify(&self) -> Result<ResponseKind, SchemaError> {
        if self.stream == Some(true) {
            if !self.ok || self.data.is_some() || self.error.is_some() {
                return Err(SchemaError::InvalidEnvelope(
                    "stream acknowledgment carries extra terminal fields".to_string(),
                ));
            }
            return Ok(ResponseKind::StreamAcknowledged);
        }
        if self.ok {
            Ok(ResponseKind::Success {
                data: self.data.clone(),
            })
        } else {
            Ok(ResponseKind::Failure {
                error: self.error.clone(),
            })
        }
    }
}
