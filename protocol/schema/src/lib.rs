//! Kitty remote-control wire schema.
//!
//! Types for the JSON envelopes exchanged over the kitty remote-control (RC)
//! protocol, as documented in kitty's `docs/rc_protocol.rst` and implemented by
//! its dispatcher (`kitty/remote_control.py`):
//!
//! - [`PlainRequest`]: the unencrypted outbound command envelope.
//! - [`ProtectedEnvelope`]: the encrypted outbound envelope produced by the
//!   one protection boundary in `kitty-rc-crypto`.
//! - [`ResponseEnvelope`] and [`ResponseKind`]: server replies, classified into
//!   terminal success, terminal failure, and streaming handshake
//!   acknowledgments.
//!
//! This crate owns no I/O, no framing, and no command semantics. Wire names are
//! upstream's: in particular the asynchronous-request field is `async` on the
//! wire (the dispatcher's `async_id` naming is internal to kitty).

mod error;
mod request;
mod response;
mod version;

pub use error::SchemaError;
pub use request::{PlainRequest, ProtectedEnvelope};
pub use response::{ResponseEnvelope, ResponseKind};
pub use version::{ProtocolVersion, SUPPORTED};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Minimal request shape from the protocol documentation example.
    #[test]
    fn plain_request_minimal_documented_shape() {
        let raw = br#"{"cmd":"ls","version":[0,14,2]}"#;
        let request: PlainRequest = serde_json::from_slice(raw).unwrap();
        assert_eq!(request.cmd, "ls");
        assert_eq!(request.version, ProtocolVersion::new([0, 14, 2]));
        assert_eq!(request.no_response, None);
        assert_eq!(request.payload, None);

        // Re-encoding a minimal request yields exactly the documented shape.
        let mut request = PlainRequest::new("ls");
        request.version = ProtocolVersion::new([0, 14, 2]);
        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"cmd":"ls","version":[0,14,2]}"#
        );
    }

    /// The async request id serializes under the wire name `async`.
    #[test]
    fn async_field_uses_wire_name() {
        let mut request = PlainRequest::new("select-window");
        request.r#async = Some("a1b2c3".to_string());
        let encoded = serde_json::to_string(&request).unwrap();
        assert!(encoded.contains(r#""async":"a1b2c3""#), "{encoded}");
        assert!(!encoded.contains("async_id"), "{encoded}");

        let decoded: PlainRequest =
            serde_json::from_str(r#"{"cmd":"select-window","version":[0,43,1],"async":"a1b2c3"}"#)
                .unwrap();
        assert_eq!(decoded.r#async.as_deref(), Some("a1b2c3"));
    }

    #[test]
    fn stream_fields_round_trip() {
        let raw =
            r#"{"cmd":"set-background-image","version":[0,43,1],"stream":true,"stream_id":"s1"}"#;
        let request: PlainRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(request.stream, Some(true));
        assert_eq!(request.stream_id.as_deref(), Some("s1"));
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({"cmd":"set-background-image","version":[0,43,1],"stream":true,"stream_id":"s1"})
        );
    }

    /// kitty_window_id is an integer on the wire (Go client sends the
    /// KITTY_WINDOW_ID env value as a number).
    #[test]
    fn kitty_window_id_is_numeric() {
        let raw = r#"{"cmd":"ls","version":[0,43,1],"kitty_window_id":3}"#;
        let request: PlainRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(request.kitty_window_id, Some(3));
    }

    /// Encrypted envelope field set per rc_protocol.rst: version, iv, tag,
    /// pubkey, encrypted (all but version are RFC 1924 base85 strings).
    #[test]
    fn protected_envelope_documented_shape() {
        // RFC 1924 base85 grows 4 bytes to 5 characters: a 12-byte IV is 15
        // characters, a 16-byte tag 20, a 32-byte key 40.
        let (iv, tag, pubkey) = ("i".repeat(15), "t".repeat(20), "p".repeat(40));
        let raw = format!(
            r#"{{"version":[0,43,1],"iv":"{iv}","tag":"{tag}","pubkey":"{pubkey}","encrypted":"e"}}"#
        );
        let envelope: ProtectedEnvelope = serde_json::from_str(&raw).unwrap();
        assert_eq!(envelope.version, ProtocolVersion::new([0, 43, 1]));
        assert_eq!(envelope.iv.len(), 15);
        assert_eq!(envelope.tag.len(), 20);
        assert_eq!(envelope.pubkey.len(), 40);

        let encoded = serde_json::to_value(&envelope).unwrap();
        for key in ["version", "iv", "tag", "pubkey", "encrypted"] {
            assert!(encoded.get(key).is_some(), "missing {key}");
        }
        assert!(encoded.get("enc_proto").is_none());
    }

    /// `enc_proto` is only emitted by upstream clients using an encryption
    /// protocol other than '1'; decoding tolerates it.
    #[test]
    fn protected_envelope_tolerates_enc_proto() {
        let raw = r#"{"version":[0,43,1],"iv":"i","tag":"t","pubkey":"p","encrypted":"e","enc_proto":"2"}"#;
        let envelope: ProtectedEnvelope = serde_json::from_str(raw).unwrap();
        assert_eq!(envelope.enc_proto.as_deref(), Some("2"));
    }

    /// Terminal responses: success with data, success without data, failure.
    #[test]
    fn response_classification_terminal() {
        let success: ResponseEnvelope =
            serde_json::from_str(r#"{"ok":true,"data":"[{\"id\":1}]"}"#).unwrap();
        assert!(matches!(
            success.classify().unwrap(),
            ResponseKind::Success { .. }
        ));

        let bare: ResponseEnvelope = serde_json::from_str(r#"{"ok":true}"#).unwrap();
        assert!(matches!(
            bare.classify().unwrap(),
            ResponseKind::Success { data: None }
        ));

        let failure: ResponseEnvelope =
            serde_json::from_str(r#"{"ok":false,"error":"nope"}"#).unwrap();
        match failure.classify().unwrap() {
            ResponseKind::Failure { error } => assert_eq!(error.as_deref(), Some("nope")),
            other => panic!("expected failure, got {other:?}"),
        }
    }

    /// Streaming handshake acknowledgment per the dispatcher: the reply to a
    /// first nonempty streaming chunk is exactly ok:true with stream:true.
    #[test]
    fn response_classification_stream_ack() {
        let ack: ResponseEnvelope = serde_json::from_str(r#"{"ok":true,"stream":true}"#).unwrap();
        assert!(matches!(
            ack.classify().unwrap(),
            ResponseKind::StreamAcknowledged
        ));

        // A stream flag paired with terminal data is not a known server shape.
        let mixed: ResponseEnvelope =
            serde_json::from_str(r#"{"ok":true,"stream":true,"data":"x"}"#).unwrap();
        assert!(mixed.classify().is_err());
    }

    /// Additive unknown fields remain tolerated on responses.
    #[test]
    fn response_tolerates_unknown_fields() {
        let raw = r#"{"ok":true,"data":1,"future_field":{"nested":true}}"#;
        let response: ResponseEnvelope = serde_json::from_str(raw).unwrap();
        assert!(response.classify().is_ok());
    }

    #[test]
    fn version_compatibility_rule() {
        // The server rejects a client whose first two components are newer.
        let client = ProtocolVersion::new([0, 44, 0]);
        let server = ProtocolVersion::new([0, 43, 1]);
        assert!(client.major_minor_newer_than(&server));
        assert!(!server.major_minor_newer_than(&client));

        // Patch differences do not trigger rejection.
        let client_patch_newer = ProtocolVersion::new([0, 43, 2]);
        let server_patch_older = ProtocolVersion::new([0, 43, 1]);
        assert!(!client_patch_newer.major_minor_newer_than(&server_patch_older));

        // The declared support target this workspace develops against.
        assert_eq!(SUPPORTED, ProtocolVersion::new([0, 43, 1]));
    }

    #[test]
    fn json_byte_round_trip() {
        let mut request = PlainRequest::new("send-text");
        request.payload = Some(json!({"data": "text:hello"}));
        request.no_response = Some(true);
        let bytes = request.to_json_bytes().unwrap();
        let decoded = PlainRequest::from_json_bytes(&bytes).unwrap();
        assert_eq!(decoded, request);
        assert_eq!(decoded.no_response, Some(true));
    }
}
