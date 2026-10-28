//! The finite core uses the existing executor-owned suppression boundary.
use kitty_rc_commands::window::SendText;
use kitty_rc_commands::{Command, Mode};
use kitty_rc_endpoint::Protection;
use kitty_rc_exchange::{Completion, Outcome, PrepareError, prepare};
use serde_json::Value;

#[test]
fn text_and_bytes_prepare_as_submission_only_without_upload_metadata() {
    for command in [SendText::new(r"literal \n"), SendText::bytes([0, 0xff])] {
        // Commands describe content; preparation, not constructors, owns flags.
        assert!(command.request().no_response.is_none());
        let prepared = prepare(&command, &Protection::Plain, 4096).unwrap();
        assert_eq!(prepared.mode(), Mode::SubmissionOnly);
        assert!(!prepared.reusable());
        let wire: Value = serde_json::from_slice(prepared.plaintext()).unwrap();
        assert_eq!(wire["cmd"], "send-text");
        assert_eq!(wire["no_response"], true);
        for absent in ["stream", "stream_id", "async"] {
            assert!(wire.get(absent).is_none(), "unexpected {absent}");
        }
        assert!(matches!(
            Outcome::Submitted.decode::<SendText>().unwrap(),
            Completion::Submitted
        ));
    }
}

#[test]
fn finite_bytes_are_not_chunked_at_the_reference_clients_1024_byte_choice() {
    let command = SendText::bytes(vec![0xff; 1025]);
    assert!(prepare(&command, &Protection::Plain, 4096).is_ok());
    // Existing caller-selected bounds still reject a large finite request.
    assert!(matches!(
        prepare(&command, &Protection::Plain, 1024),
        Err(PrepareError::Limit)
    ));
}
