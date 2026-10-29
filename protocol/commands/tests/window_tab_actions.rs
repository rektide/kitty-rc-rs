//! Wire goldens for the window/tab helpers, process creation with inherited
//! working directory, and named clipboard buffers. The opt-in companion test
//! executes kitty's real full parser on Rust-generated strings.
use kitty_rc_commands::actions::{
    Action, BufferName, InvalidBufferName, KnownAction, NewProcess, SimpleAction,
};
use kitty_rc_commands::targets::{WindowId, WindowTarget};
use kitty_rc_commands::{Class, Command, Mode, Submission};
use serde_json::json;

const RAW_REMAINDER_NAMES: [&str; 15] = [
    "scratch",
    "my buffer",
    "two  spaces",
    "tab\tname",
    "line\nname",
    r"a\b",
    "a\\b",
    "a'b\"c",
    "$HOME \\ ~ x",
    "a\u{85}b",
    "a\u{a0}b",
    "a\0b",
    "clipboard",
    "primary",
    "🦇buf",
];

#[test]
fn window_tab_and_clipboard_helpers_use_native_zero_argument_names() {
    let cases = [
        (
            SimpleAction::CloseOtherWindowsInTab,
            "close_other_windows_in_tab",
        ),
        (
            SimpleAction::CloseOtherTabsInOsWindow,
            "close_other_tabs_in_os_window",
        ),
        (SimpleAction::MoveWindowToTop, "move_window_to_top"),
        (SimpleAction::CopyAnsiToClipboard, "copy_ansi_to_clipboard"),
        (
            SimpleAction::CopyLastCommandOutput,
            "copy_last_command_output",
        ),
        (
            SimpleAction::ShowLastCommandOutput,
            "show_last_command_output",
        ),
    ];
    for (action, name) in cases {
        assert_eq!(action.as_str(), name);
        assert_eq!(KnownAction::Simple(action).wire(), name);
        let request = Action::new(action).request();
        assert_eq!(request.cmd, "action");
        assert_eq!(
            request.payload,
            Some(json!({"action": name, "match_window": null, "self": false}))
        );
    }
}

#[test]
fn cwd_process_kinds_share_the_shell_split_argv_grammar() {
    // options/utils.py shlex_parse: new_tab_with_cwd and new_os_window_with_cwd
    // use to_cmdline exactly like the existing new_* kinds, then source the
    // child cwd from the dispatch window's shell-integration reporting.
    for (kind, name) in [
        (NewProcess::TabWithCwd, "new_tab_with_cwd"),
        (NewProcess::OsWindowWithCwd, "new_os_window_with_cwd"),
    ] {
        assert_eq!(
            KnownAction::NewProcess {
                kind,
                args: vec!["echo".into(), "a b".into(), "it's".into()],
            }
            .wire(),
            format!("{name} 'echo' 'a b' 'it'\"'\"'s'")
        );
    }
    // Family consistency with the pre-existing cwd kind.
    assert_eq!(
        KnownAction::NewProcess {
            kind: NewProcess::WindowWithCwd,
            args: vec!["sh".into()],
        }
        .wire(),
        "new_window_with_cwd 'sh'"
    );
}

#[test]
fn buffer_names_stay_a_literal_raw_remainder() {
    // options/utils.py copy_to_buffer/paste_from_buffer: `return func, [rest]`.
    // No shell quoting and no escape encoding: spaces, tabs, newlines,
    // backslashes, quotes and shell metacharacters stay one literal name.
    for name in RAW_REMAINDER_NAMES {
        let buffer = BufferName::new(name).unwrap();
        assert_eq!(buffer.as_str(), name);
        assert_eq!(
            KnownAction::CopyToBuffer(buffer.clone()).wire(),
            format!("copy_to_buffer {name}")
        );
        assert_eq!(
            KnownAction::PasteFromBuffer(buffer).wire(),
            format!("paste_from_buffer {name}")
        );
        assert_eq!(
            KnownAction::copy_to_buffer(name).unwrap().wire(),
            format!("copy_to_buffer {name}")
        );
        assert_eq!(
            KnownAction::paste_from_buffer(name).unwrap().wire(),
            format!("paste_from_buffer {name}")
        );
    }
}

#[test]
fn buffer_names_reject_empty_and_parser_altered_edges_only() {
    assert_eq!(BufferName::new(""), Err(InvalidBufferName::Empty));
    // Leading whitespace joins the parser's separator run; trailing
    // whitespace (including the \u{1c}–\u{1f} controls and NEL) is stripped
    // by the outer strip. Such names would address a different buffer.
    for altered in [
        " ",
        "\t",
        "  x",
        "x ",
        "x\t",
        "\nx",
        "x\r",
        "x\u{b}",
        "\x1cx",
        "x\x1f",
        "\u{a0}x",
        "x\u{2028}",
        "\u{85}",
        "x\u{85}",
    ] {
        assert_eq!(
            BufferName::new(altered),
            Err(InvalidBufferName::EdgeWhitespace),
            "expected edge rejection for {altered:?}"
        );
    }
    // Interior characters of every one of those classes stay representable.
    for literal in [
        "a b",
        "a\tb",
        "a\nb",
        "a\rb",
        "a\u{b}b",
        "a\u{1c}b",
        "a\u{1f}b",
        "a\u{a0}b",
        "a\u{2028}b",
        "a\u{85}b",
    ] {
        let buffer = BufferName::new(literal).unwrap_or_else(|error| {
            panic!("expected interior acceptance for {literal:?}: {error}")
        });
        assert_eq!(buffer.as_str(), literal);
    }
    assert!(matches!(
        KnownAction::copy_to_buffer(" "),
        Err(InvalidBufferName::EdgeWhitespace)
    ));
    assert!(matches!(
        KnownAction::paste_from_buffer(""),
        Err(InvalidBufferName::Empty)
    ));
    assert_eq!(
        InvalidBufferName::Empty.to_string(),
        "clipboard buffer name must not be empty"
    );
    assert_eq!(
        InvalidBufferName::EdgeWhitespace.to_string(),
        "clipboard buffer name must not start or end with whitespace"
    );
}

#[test]
fn helper_and_buffer_actions_preserve_targets_and_dispatch_only_contract() {
    for (target, match_window, only_self) in [
        (WindowTarget::Active, json!(null), false),
        (WindowTarget::CallingWindow, json!(null), true),
        (WindowTarget::Id(WindowId(17)), json!("id:17"), false),
    ] {
        for known in [
            KnownAction::Simple(SimpleAction::CloseOtherWindowsInTab),
            KnownAction::NewProcess {
                kind: NewProcess::TabWithCwd,
                args: vec!["sh".into()],
            },
            KnownAction::copy_to_buffer("my buffer").unwrap(),
            KnownAction::paste_from_buffer("primary").unwrap(),
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
    // Successful dispatch has no required selection/clipboard receipt.
    assert!(Action::decode(None).is_ok());
    assert!(Action::decode(Some(json!({"not_a_receipt": true}))).is_ok());
}

/// Explicit executable only: no default PATH lookup, download, terminal
/// server, config mutation or mandatory native dependency for normal tests.
#[test]
#[ignore = "set KITTY_RC_NATIVE_KITTY to a native kitty executable; parser-only evidence"]
fn native_full_parser_preserves_rust_generated_helper_actions() {
    use std::io::Write;
    use std::process::{Command as ProcessCommand, Stdio};

    let executable = std::env::var_os("KITTY_RC_NATIVE_KITTY")
        .expect("set KITTY_RC_NATIVE_KITTY to the explicit kitty executable");
    let mut cases = Vec::new();
    let zero_argument: [(SimpleAction, &str); 6] = [
        (
            SimpleAction::CloseOtherWindowsInTab,
            "close_other_windows_in_tab",
        ),
        (
            SimpleAction::CloseOtherTabsInOsWindow,
            "close_other_tabs_in_os_window",
        ),
        (SimpleAction::MoveWindowToTop, "move_window_to_top"),
        (SimpleAction::CopyAnsiToClipboard, "copy_ansi_to_clipboard"),
        (
            SimpleAction::CopyLastCommandOutput,
            "copy_last_command_output",
        ),
        (
            SimpleAction::ShowLastCommandOutput,
            "show_last_command_output",
        ),
    ];
    for (action, name) in zero_argument {
        cases.push(json!({
            "func": name, "args": [],
            "wire": KnownAction::Simple(action).wire(),
        }));
    }
    for name in RAW_REMAINDER_NAMES {
        for func in ["copy_to_buffer", "paste_from_buffer"] {
            let known = if func == "copy_to_buffer" {
                KnownAction::copy_to_buffer(name).unwrap()
            } else {
                KnownAction::paste_from_buffer(name).unwrap()
            };
            cases.push(json!({"func": func, "args": [name], "wire": known.wire()}));
        }
    }
    for (kind, name, args) in [
        (
            NewProcess::TabWithCwd,
            "new_tab_with_cwd",
            vec!["echo".to_owned(), "a b".to_owned()],
        ),
        (
            NewProcess::OsWindowWithCwd,
            "new_os_window_with_cwd",
            vec!["it's".to_owned()],
        ),
    ] {
        let known = KnownAction::NewProcess {
            kind,
            args: args.clone(),
        };
        cases.push(json!({"func": name, "args": args, "wire": known.wire()}));
    }

    let mut child = ProcessCommand::new(executable)
        .arg("+runpy")
        .arg(include_str!("fixtures/window_tab_actions.py"))
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
    assert_eq!(report["cases"], cases.len());
    assert_eq!(report["edge_shapes_checked"], 4);
    eprintln!("{report}");
}
