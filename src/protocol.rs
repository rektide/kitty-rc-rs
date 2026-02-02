use crate::error::ProtocolError;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

const PREFIX: &str = "\x1bP@kitty-cmd";
const SUFFIX: &str = "\x1b\\";
const MAX_CHUNK_SIZE: usize = 4096;

static STREAM_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KittyMessage {
    pub cmd: String,
    pub version: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_response: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kitty_window_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl KittyMessage {
    pub fn new(cmd: impl Into<String>, version: impl Into<Vec<u32>>) -> Self {
        Self {
            cmd: cmd.into(),
            version: version.into(),
            no_response: None,
            kitty_window_id: None,
            payload: None,
            async_id: None,
            cancel_async: None,
            stream_id: None,
            stream: None,
        }
    }

    pub fn no_response(mut self, value: bool) -> Self {
        self.no_response = Some(value);
        self
    }

    pub fn kitty_window_id(mut self, id: impl Into<String>) -> Self {
        self.kitty_window_id = Some(id.into());
        self
    }

    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn async_id(mut self, id: impl Into<String>) -> Self {
        self.async_id = Some(id.into());
        self
    }

    pub fn cancel_async(mut self, value: bool) -> Self {
        self.cancel_async = Some(value);
        self
    }

    pub fn stream_id(mut self, id: impl Into<String>) -> Self {
        self.stream_id = Some(id.into());
        self
    }

    pub fn stream(mut self, value: bool) -> Self {
        self.stream = Some(value);
        self
    }

    pub fn generate_unique_id() -> String {
        let id = STREAM_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{:x}", id)
    }

    pub fn needs_streaming(&self) -> bool {
        if let Some(payload) = &self.payload {
            if let Some(obj) = payload.as_object() {
                for (_key, value) in obj {
                    if let Some(s) = value.as_str() {
                        if s.len() > MAX_CHUNK_SIZE {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn into_chunks(mut self) -> Vec<KittyMessage> {
        let mut chunks = Vec::new();

        if !self.needs_streaming() {
            return vec![self];
        }

        if let Some(payload) = self.payload.take() {
            if let Some(obj) = payload.as_object() {
                let stream_id = Self::generate_unique_id();

                for (_key, value) in obj {
                    if let Some(s) = value.as_str() {
                        if s.len() > MAX_CHUNK_SIZE {
                            for (i, chunk_data) in s.as_bytes().chunks(MAX_CHUNK_SIZE).enumerate() {
                                let mut chunk_msg = self.clone();
                                chunk_msg.stream_id = Some(stream_id.clone());
                                chunk_msg.stream = Some(true);

                                let mut chunk_payload = serde_json::Map::new();
                                chunk_payload.insert(
                                    "data".to_string(),
                                    serde_json::Value::String(
                                        String::from_utf8_lossy(chunk_data).to_string(),
                                    ),
                                );
                                chunk_payload.insert("chunk_num".to_string(), serde_json::json!(i));
                                chunk_msg.payload = Some(serde_json::Value::Object(chunk_payload));

                                chunks.push(chunk_msg);
                            }

                            let mut end_chunk = self.clone();
                            end_chunk.stream_id = Some(stream_id);
                            end_chunk.stream = Some(true);
                            let mut end_payload = serde_json::Map::new();
                            end_payload.insert(
                                "data".to_string(),
                                serde_json::Value::String(String::new()),
                            );
                            end_chunk.payload = Some(serde_json::Value::Object(end_payload));
                            chunks.push(end_chunk);

                            return chunks;
                        }
                    }
                }
            }
        }

        chunks.push(self);
        chunks
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let json = serde_json::to_string(self)?;
        let message = format!("{}{}{}", PREFIX, json, SUFFIX);
        Ok(message.into_bytes())
    }

    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        let s = std::str::from_utf8(data)
            .map_err(|e| ProtocolError::InvalidMessageFormat(e.to_string()))?;

        if !s.starts_with(PREFIX) {
            return Err(ProtocolError::InvalidEscapeSequence);
        }

        if !s.ends_with(SUFFIX) {
            return Err(ProtocolError::InvalidEscapeSequence);
        }

        let json_start = PREFIX.len();
        let json_end = s.len() - SUFFIX.len();
        let json_str = &s[json_start..json_end];

        serde_json::from_str(json_str).map_err(ProtocolError::JsonError)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KittyResponse {
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl KittyResponse {
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        let s = std::str::from_utf8(data)
            .map_err(|e| ProtocolError::EnvelopeParseError(e.to_string()))?;

        if !s.starts_with("\x1bP@kitty-cmd") {
            return Err(ProtocolError::EnvelopeParseError(
                "Invalid response prefix".to_string(),
            ));
        }

        if !s.ends_with("\x1b\\") {
            return Err(ProtocolError::EnvelopeParseError(
                "Invalid response suffix".to_string(),
            ));
        }

        let json_start = PREFIX.len();
        let json_end = s.len() - SUFFIX.len();
        let json_str = &s[json_start..json_end];

        let msg: serde_json::Value =
            serde_json::from_str(json_str).map_err(ProtocolError::JsonError)?;

        if !msg.is_object() {
            return Err(ProtocolError::EnvelopeParseError(
                "Response is not a JSON object".to_string(),
            ));
        }

        serde_json::from_value(msg).map_err(ProtocolError::JsonError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_encode() {
        let msg = KittyMessage::new("ls", vec![0, 14, 2]);
        let encoded = msg.encode().unwrap();
        let decoded = KittyMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.cmd, "ls");
        assert_eq!(decoded.version, vec![0, 14, 2]);
    }

    #[test]
    fn test_message_with_payload() {
        let msg = KittyMessage::new("send-text", vec![0, 14, 2])
            .payload(serde_json::json!({"match": "id:1", "data": "text:hello"}));
        let encoded = msg.encode().unwrap();
        let decoded = KittyMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.cmd, "send-text");
        assert!(decoded.payload.is_some());
    }

    #[test]
    fn test_message_no_response() {
        let msg = KittyMessage::new("close-window", vec![0, 14, 2]).no_response(true);
        let encoded = msg.encode().unwrap();
        let decoded = KittyMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.no_response, Some(true));
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let data = b"invalid message";
        let result = KittyMessage::decode(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_response_decode() {
        let raw = b"\x1bP@kitty-cmd{\"ok\":true,\"data\":[{\"id\":1,\"title\":\"test\"}]}\x1b\\";
        let response = KittyResponse::decode(raw).unwrap();
        assert!(response.ok);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_async_id() {
        let msg = KittyMessage::new("select-window", vec![0, 14, 2]).async_id("abc123");
        let encoded = msg.encode().unwrap();
        let decoded = KittyMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.async_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_cancel_async() {
        let msg = KittyMessage::new("select-window", vec![0, 14, 2])
            .async_id("abc123")
            .cancel_async(true);
        let encoded = msg.encode().unwrap();
        let decoded = KittyMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.cancel_async, Some(true));
    }

    #[test]
    fn test_unique_id_generation() {
        let id1 = KittyMessage::generate_unique_id();
        let id2 = KittyMessage::generate_unique_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_needs_streaming_false() {
        let msg = KittyMessage::new("send-text", vec![0, 14, 2])
            .payload(serde_json::json!({"data": "hello"}));
        assert!(!msg.needs_streaming());
    }

    #[test]
    fn test_needs_streaming_true() {
        let large_data = "x".repeat(5000);
        let msg = KittyMessage::new("send-text", vec![0, 14, 2])
            .payload(serde_json::json!({"data": large_data}));
        assert!(msg.needs_streaming());
    }

    #[test]
    fn test_into_chunks_no_streaming() {
        let msg = KittyMessage::new("send-text", vec![0, 14, 2])
            .payload(serde_json::json!({"data": "hello"}));
        let chunks = msg.into_chunks();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_into_chunks_with_streaming() {
        let large_data = "x".repeat(5000);
        let msg = KittyMessage::new("set-background-image", vec![0, 14, 2])
            .payload(serde_json::json!({"data": large_data}));
        let chunks = msg.into_chunks();
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.stream_id.is_some()));
        assert!(chunks.iter().all(|c| c.stream == Some(true)));
    }
}
