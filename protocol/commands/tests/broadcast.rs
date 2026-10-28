//! Wire contracts read from both catalog-pinned `send_text.py` revisions.
use kitty_rc_commands::Command;
use kitty_rc_commands::targets::{TabId, WindowId, WindowSet};
use kitty_rc_commands::window::{
    BracketedPaste, BroadcastControl, BroadcastId, BroadcastOperation, SendText,
};
use serde_json::{Value, json};

fn payload(command: &impl Command) -> Value {
    let request = command.request();
    assert_eq!(request.cmd, "send-text");
    request.payload.unwrap()
}

#[test]
fn broadcast_id_rejects_only_empty_without_normalizing() {
    assert!(BroadcastId::new("").is_err());
    for raw in [" ", " \tcaller:\0🦀\n "] {
        let id = BroadcastId::new(raw).unwrap();
        assert_eq!(id.as_str(), raw);
        assert_eq!(json!(id), raw);
    }
}

#[test]
fn explicit_start_data_end_share_identity_and_preserve_scope_and_exclusion() {
    let id = BroadcastId::new("caller-selected").unwrap();
    let windows = WindowSet::builder()
        .match_window(WindowId(7))
        .match_tab(TabId(3))
        .all(true)
        .build();
    let start = BroadcastControl {
        windows: windows.clone(),
        exclude_active: true,
        ..BroadcastControl::start(id.clone())
    };
    let text = SendText::builder()
        .data(r"literal \n")
        .broadcast(id.clone())
        .windows(windows.clone())
        .exclude_active(true)
        .bracketed_paste(BracketedPaste::Auto)
        .build();
    let bytes = SendText {
        broadcast: Some(id.clone()),
        windows: windows.clone(),
        exclude_active: true,
        ..SendText::bytes([0, 0xff])
    };
    let end = BroadcastControl::builder()
        .broadcast(id)
        .operation(BroadcastOperation::End)
        .windows(windows)
        .exclude_active(true)
        .build();
    for (actual, data, paste) in [
        (payload(&start), "session:start", None),
        (payload(&text), r"text:literal \n", Some("auto")),
        (payload(&bytes), "base64:AP8=", Some("disable")),
        (payload(&end), "session:end", None),
    ] {
        let mut expected = json!({
            "data": data, "session_id": "caller-selected",
            "match": "id:7", "match_tab": "id:3", "all": true, "exclude_active": true
        });
        if let Some(paste) = paste {
            expected["bracketed_paste"] = json!(paste);
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn ordinary_data_has_no_implicit_broadcast_and_controls_default_only_scope() {
    for command in [
        SendText::default(),
        SendText::new("literal"),
        SendText::bytes([0, 0xff]),
        SendText::builder().data("literal").build(),
    ] {
        assert!(payload(&command).get("session_id").is_none());
    }
    let id = BroadcastId::new("explicit").unwrap();
    assert_eq!(
        payload(&BroadcastControl::end(id)),
        json!({
            "data": "session:end", "session_id": "explicit",
            "match": null, "match_tab": null, "all": false, "exclude_active": false
        })
    );
}
