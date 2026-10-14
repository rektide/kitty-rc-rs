use crate::error::SchemaError;
use crate::version::{ProtocolVersion, SUPPORTED};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An unencrypted outbound command envelope.
///
/// Wire shape per `docs/rc_protocol.rst`: `cmd`, array `version`, optional
/// `no_response`, `kitty_window_id`, `payload`, `async`, `cancel_async`,
/// `stream_id`, and `stream`. Field names here are upstream's, including the
/// `async` rename (kitty calls it `async_id` only after decoding).
///
/// This type is deliberately distinct from [`ProtectedEnvelope`]: a plain
/// request is transformed into a protected envelope exactly once, by
/// `kitty-rc-crypto`, and transport code never sees a value it could encrypt
/// again.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainRequest {
    pub cmd: String,
    pub version: ProtocolVersion,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_response: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kitty_window_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(default, rename = "async", skip_serializing_if = "Option::is_none")]
    pub r#async: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_async: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl PlainRequest {
    /// A plain request for `cmd` at the workspace's declared supported
    /// version. All optional fields start absent.
    pub fn new(cmd: impl Into<String>) -> Self {
        Self {
            cmd: cmd.into(),
            version: SUPPORTED,
            no_response: None,
            kitty_window_id: None,
            payload: None,
            r#async: None,
            cancel_async: None,
            stream_id: None,
            stream: None,
        }
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, SchemaError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, SchemaError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// An encrypted outbound envelope: what actually goes on the wire when a plain
/// request is protected.
///
/// Wire shape per `docs/rc_protocol.rst`: array `version` plus the RFC 1924
/// base85 fields `iv` (>= 96 bits of CSPRNG), `tag` (>= 128 bits), `pubkey`
/// (sender X25519 public key), and `encrypted` (the original command, with
/// `password` and `timestamp` injected at its top level, under AES-256-GCM).
/// `enc_proto` is emitted by upstream only when the encryption protocol is not
/// `'1'`; decoding tolerates its presence.
///
/// Construction is owned by `kitty-rc-crypto`; this type is the wire schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedEnvelope {
    pub version: ProtocolVersion,
    pub iv: String,
    pub tag: String,
    pub pubkey: String,
    pub encrypted: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enc_proto: Option<String>,
}

impl ProtectedEnvelope {
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, SchemaError> {
        Ok(serde_json::to_vec(self)?)
    }
}
