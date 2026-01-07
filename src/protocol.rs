use crate::error::ProtocolError;
use serde::{Deserialize, Serialize};

const PREFIX: &str = "\x1bP@kitty-cmd";
const SUFFIX: &str = "\x1b\\";

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
}

impl KittyMessage {
    pub fn new(cmd: impl Into<String>, version: impl Into<Vec<u32>>) -> Self {
        Self {
            cmd: cmd.into(),
            version: version.into(),
            no_response: None,
            kitty_window_id: None,
            payload: None,
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

        let msg: serde_json::Value = serde_json::from_str(json_str)
            .map_err(ProtocolError::JsonError)?;

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
        let msg = KittyMessage::new("close-window", vec![0, 14, 2])
            .no_response(true);
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
}
