//! Typed nested text keeps the existing action acknowledgment/retirement path.
use kitty_rc_commands::actions::{Action, KnownAction, SendTextModes};
use kitty_rc_commands::{Mode, Submission};
use kitty_rc_endpoint::Protection;
use kitty_rc_exchange::{Completion, Failure, Outcome, prepare};
use serde_json::Value;

#[test]
fn nested_text_preparation_retires_with_or_without_dispatch_reply() {
    for known in [
        KnownAction::paste(r"\n").unwrap(),
        KnownAction::send_text(SendTextModes::All, r"\n").unwrap(),
    ] {
        let action = Action::new(known);
        let prepared = prepare(&action, &Protection::Plain, 4096).unwrap();
        assert_eq!(prepared.mode(), Mode::Reply);
        assert!(!prepared.reusable());
        let wire: Value = serde_json::from_slice(prepared.plaintext()).unwrap();
        assert_eq!(wire["cmd"], "action");
        for absent in ["no_response", "stream", "stream_id", "async"] {
            assert!(wire.get(absent).is_none(), "unexpected {absent}");
        }

        let prepared = prepare(&Submission(action), &Protection::Plain, 4096).unwrap();
        assert_eq!(prepared.mode(), Mode::SubmissionOnly);
        assert!(!prepared.reusable());
        let suppressed: Value = serde_json::from_slice(prepared.plaintext()).unwrap();
        assert_eq!(suppressed["payload"], wire["payload"]);
        assert_eq!(suppressed["no_response"], true);
    }
}

#[test]
fn dispatch_ack_is_unit_and_remote_mode_mismatch_is_not_fabricated_success() {
    assert!(matches!(
        Outcome::Reply(br#"{"ok":true}"#.to_vec())
            .decode::<Action>()
            .unwrap(),
        Completion::Reply(())
    ));
    assert!(matches!(
        Outcome::Submitted.decode::<Submission<Action>>().unwrap(),
        Completion::Submitted
    ));
    // A known native action may be unconsumed (e.g. a send_text mode mismatch).
    let error = "Unknown action: send_text normal \\U00000061";
    let reply = serde_json::to_vec(&serde_json::json!({"ok": false, "error": error})).unwrap();
    assert!(matches!(
        Outcome::Reply(reply).decode::<Action>(),
        Err(Failure::Remote { error: actual, .. }) if actual == error
    ));
}
