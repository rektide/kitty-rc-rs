//! Independent full-wire goldens for literal nested text, not a local decoder
//! roundtrip. The opt-in companion test executes kitty's real full parser.
use kitty_rc_commands::actions::{
    Action, ActionSpec, ActionText, EmptyActionText, KnownAction, SendTextModes,
};
use kitty_rc_commands::targets::{WindowId, WindowMatch, WindowTarget};
use kitty_rc_commands::{Class, Command, Mode, Submission};
use serde_json::json;

#[test]
fn nested_literal_wire_goldens_preserve_whitespace_backslashes_and_scalars() {
    let cases = [
        (" x ", r"\U00000020\U00000078\U00000020"),
        (r"\n", r"\U0000005c\U0000006e"),
        ("\n", r"\U0000000a"),
        (r"\123", r"\U0000005c\U00000031\U00000032\U00000033"),
        (
            r"\uD800",
            r"\U0000005c\U00000075\U00000044\U00000038\U00000030\U00000030",
        ),
        (r"\x", r"\U0000005c\U00000078"),
        ("\\", r"\U0000005c"),
        ("\0\x1b\x7f", r"\U00000000\U0000001b\U0000007f"),
        ("\t\r\n", r"\U00000009\U0000000d\U0000000a"),
        ("\u{85}\u{2003}\u{3000}", r"\U00000085\U00002003\U00003000"),
        (
            "\u{d7ff}\u{e000}\u{ffff}\u{10000}\u{10ffff}",
            r"\U0000d7ff\U0000e000\U0000ffff\U00010000\U0010ffff",
        ),
        ("🦀e\u{301}", r"\U0001f980\U00000065\U00000301"),
    ];
    for (literal, encoded) in cases {
        let paste = Action::new(KnownAction::paste(literal).unwrap());
        let send = Action::new(KnownAction::send_text(SendTextModes::All, literal).unwrap());
        assert_eq!(paste.request().cmd, "action");
        assert_eq!(send.request().cmd, "action");
        assert_eq!(
            paste.request().payload,
            Some(json!({
                "action": format!("paste {encoded}"), "match_window": null, "self": false,
            }))
        );
        assert_eq!(
            send.request().payload,
            Some(json!({
                "action": format!("send_text all {encoded}"), "match_window": null, "self": false,
            }))
        );
    }
}

#[test]
fn text_invariant_rejects_only_empty_and_public_variants_require_validated_text() {
    assert_eq!(ActionText::new(""), Err(EmptyActionText));
    assert!(matches!(KnownAction::paste(""), Err(EmptyActionText)));
    assert!(matches!(
        KnownAction::send_text(SendTextModes::All, ""),
        Err(EmptyActionText)
    ));
    assert_eq!(
        EmptyActionText.to_string(),
        "nested action text must not be empty"
    );

    for literal in [" ", "\t\r\n", "\0", r"\n", "\u{10ffff}"] {
        let text = ActionText::new(literal.to_owned()).unwrap();
        assert_eq!(text.as_str(), literal);
        let clone = text.clone();
        assert_eq!(clone, text);
        assert_eq!(
            KnownAction::Paste(text.clone()).wire(),
            KnownAction::paste(literal).unwrap().wire()
        );
        assert_eq!(
            KnownAction::SendText {
                modes: SendTextModes::Kitty,
                text
            }
            .wire(),
            KnownAction::send_text(SendTextModes::Kitty, literal)
                .unwrap()
                .wire()
        );
    }
}

const MODE_CASES: [(SendTextModes, &str); 8] = [
    (SendTextModes::All, "all"),
    (SendTextModes::Normal, "normal"),
    (SendTextModes::Application, "application"),
    (SendTextModes::Kitty, "kitty"),
    (SendTextModes::NormalApplication, "normal,application"),
    (SendTextModes::NormalKitty, "normal,kitty"),
    (SendTextModes::ApplicationKitty, "application,kitty"),
    (
        SendTextModes::NormalApplicationKitty,
        "normal,application,kitty",
    ),
];

#[test]
fn every_nonempty_mode_combination_has_a_canonical_wire_spelling() {
    for (modes, spelling) in MODE_CASES {
        assert_eq!(modes.as_str(), spelling);
        assert_eq!(
            KnownAction::send_text(modes, "a").unwrap().wire(),
            format!(r"send_text {spelling} \U00000061")
        );
    }
}

#[test]
fn typed_text_preserves_all_outer_targets_and_dispatch_only_contract() {
    for (target, match_window, only_self) in [
        (WindowTarget::Active, json!(null), false),
        (WindowTarget::CallingWindow, json!(null), true),
        (WindowTarget::Id(WindowId(17)), json!("id:17"), false),
        (
            WindowTarget::window(WindowMatch::title("editor")),
            json!("title:editor"),
            false,
        ),
    ] {
        for known in [
            KnownAction::paste("a").unwrap(),
            KnownAction::send_text(SendTextModes::Normal, "a").unwrap(),
        ] {
            let wire = known.wire();
            let action = Action::builder()
                .action(known)
                .target(target.clone())
                .build();
            assert_eq!(
                action.request().payload,
                Some(json!({"action": wire, "match_window": match_window, "self": only_self}))
            );
            assert_eq!(action.class(), Class::Short);
            assert_eq!(action.mode(), Mode::Reply);
            assert!(!action.reusable());
            let submitted = Submission(action);
            assert_eq!(submitted.mode(), Mode::SubmissionOnly);
            assert!(!submitted.reusable());
        }
    }
    // Successful dispatch has no required child-delivery payload.
    assert!(Action::decode(None).is_ok());
    assert!(Action::decode(Some(json!({"not_a_receipt": true}))).is_ok());
}

#[test]
fn raw_native_escape_source_is_unchanged() {
    let raw = "paste \\n ";
    let action = Action::new(ActionSpec::Raw(raw.into()));
    assert_eq!(action.request().payload.unwrap()["action"], raw);
    assert!(!action.reusable());
}

/// Explicit executable only: no default PATH lookup, download, terminal server,
/// config mutation or mandatory native dependency for normal unit tests.
#[test]
#[ignore = "set KITTY_RC_NATIVE_KITTY to a native kitty executable; parser-only evidence"]
fn native_full_parser_preserves_rust_generated_nested_text() {
    use std::io::Write;
    use std::process::{Command as ProcessCommand, Stdio};

    let executable = std::env::var_os("KITTY_RC_NATIVE_KITTY")
        .expect("set KITTY_RC_NATIVE_KITTY to the explicit kitty executable");
    let mut literals: Vec<String> = (0..128)
        .map(|value| char::from(value).to_string())
        .collect();
    literals.extend(
        [
            0x85, 0xa0, 0xff, 0x100, 0x1680, 0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005,
            0x2006, 0x2007, 0x2008, 0x2009, 0x200a, 0x2028, 0x2029, 0x202f, 0x205f, 0x3000, 0xd7ff,
            0xe000, 0xffff, 0x10000, 0x1f980, 0x10ffff,
        ]
        .map(|value| char::from_u32(value).unwrap().to_string()),
    );
    literals.extend(
        [
            r"\n",
            r"\123",
            r"\uD800",
            " leading and trailing ",
            "\t\r\n",
            "\0\x1b[201~",
            "abc\\",
            "'\" : ; | $HOME",
            "🦀e\u{301}",
            " \0\\n\u{10ffff} ",
        ]
        .map(str::to_owned),
    );

    let mut cases = Vec::new();
    for literal in &literals {
        cases.push(json!({
            "func": "paste", "literal": literal,
            "wire": KnownAction::paste(literal).unwrap().wire(),
        }));
        for (modes, spelling) in MODE_CASES {
            cases.push(json!({
                "func": "send_text", "modes": spelling, "literal": literal,
                "wire": KnownAction::send_text(modes, literal).unwrap().wire(),
            }));
        }
    }

    let mut child = ProcessCommand::new(executable)
        .arg("+runpy")
        .arg(include_str!("fixtures/nested_text.py"))
        .env_remove("KITTY_LISTEN_ON")
        .env_remove("KITTY_PUBLIC_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start explicit native kitty +runpy");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&cases).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "native parser failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("native parser report must be JSON");
    assert_eq!(report["paste_cases"], literals.len());
    assert_eq!(report["send_text_cases"], literals.len() * MODE_CASES.len());
    assert_eq!(report["empty_native_shapes_checked"], 2);
    eprintln!("{report}");
}
