//! Validate a native reply's shape without materializing untrusted success data
//! into a serde Value tree. Managed custody owns bounded wire bytes; only the
//! caller's sole typed take allocates the decoded output.
use kitty_rc_schema::SchemaError;
use serde::{
    Deserialize, Deserializer,
    de::{IgnoredAny, Visitor},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseHead {
    Success,
    Failure,
    StreamAcknowledged,
}
struct DiscardText;
impl<'de> Deserialize<'de> for DiscardText {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Text;
        impl<'de> Visitor<'de> for Text {
            type Value = DiscardText;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("error string")
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Self::Value, E> {
                Ok(DiscardText)
            }
        }
        d.deserialize_str(Text)
    }
}
pub fn response_head(bytes: &[u8]) -> Result<ResponseHead, SchemaError> {
    #[derive(Deserialize)]
    struct Header {
        ok: bool,
        stream: Option<bool>,
        data: Option<IgnoredAny>,
        error: Option<DiscardText>,
    }
    let h: Header = serde_json::from_slice(bytes)?;
    if h.stream == Some(true) {
        if !h.ok || h.data.is_some() || h.error.is_some() {
            return Err(SchemaError::InvalidEnvelope(
                "stream acknowledgment carries terminal fields".into(),
            ));
        }
        Ok(ResponseHead::StreamAcknowledged)
    } else if h.ok {
        Ok(ResponseHead::Success)
    } else {
        Ok(ResponseHead::Failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_header_matches_schema_without_expanding_success_data() {
        for (wire, expected) in [
            ("{\"ok\":true}", ResponseHead::Success),
            (
                "{\"ok\":false,\"error\":\"denied\\n🦀\"}",
                ResponseHead::Failure,
            ),
            (
                "{\"ok\":true,\"stream\":true}",
                ResponseHead::StreamAcknowledged,
            ),
        ] {
            assert_eq!(response_head(wire.as_bytes()).unwrap(), expected);
        }
        for wire in [
            "{}",
            "{\"ok\":true,\"stream\":true,\"data\":[]}",
            "{\"ok\":false,\"error\":1}",
            "{\"ok\":false,\"stream\":true}",
        ] {
            assert!(response_head(wire.as_bytes()).is_err());
        }
        let data = format!(
            "{{\"ok\":true,\"data\":[{}]}}",
            vec!["[]"; 100_000].join(",")
        );
        assert_eq!(
            response_head(data.as_bytes()).unwrap(),
            ResponseHead::Success
        );
    }
}
