//! Golden requests from the audited kitty v0.49.2 RC handlers and matching
//! grammar (kitty/rc/base.py and kitty/search_query_parser.py). Key spelling
//! follows kitty/options/utils.py::parse_shortcut and kitty/key_names.py;
//! signals follow kitty/rc/signal_child.py and options/utils.py::signal_child_parse.
use kitty_rc_commands::actions::{Action, KnownAction, SimpleAction};
use kitty_rc_commands::tab::{TabCommand, TabDetachDestination, TabOperation};
use kitty_rc_commands::targets::{
    Key, KeyChord, Layout, Modifiers, Signal, TabId, TabMatch, TabTarget, WindowId, WindowMatch,
    WindowSet, WindowTarget,
};
use kitty_rc_commands::window::{
    ResizeAxis, ResizeWindow, ScrollAmount, SelectWindow, SendKey, SendText, UserVars,
    WindowCommand, WindowOperation,
};
use kitty_rc_commands::{Command, FocusWindow, GetText, LaunchWait, Ls, TextExtent};
use serde_json::{Value, json};

fn payload(command: &impl Command) -> Value {
    command.request().payload.unwrap()
}

#[test]
fn selectors_encode_native_fields_and_preserve_query_boundaries() {
    assert_eq!(json!(WindowMatch::id(WindowId(7))), "id:7");
    assert_eq!(json!(TabMatch::id(TabId(2))), "id:2");
    assert_eq!(json!(WindowMatch::title("work")), "title:work");
    assert_eq!(json!(TabMatch::window_id(WindowId(7))), "window_id:7");
    assert_eq!(WindowMatch::pid(123).to_string(), "pid:123");
    assert_eq!(TabMatch::pid(123).as_str(), "pid:123");
    // The native search parser unescapes backslashes. Parentheses/spaces must
    // remain inside a single query; they must not become Boolean grammar.
    assert_eq!(
        WindowMatch::title(r#"^(work|build) "x"\d+$"#).as_str(),
        r#"title:"^(work|build) \"x\"\\d+$""#
    );
    assert_eq!(
        TabMatch::cwd("/work/my project").as_str(),
        "cwd:\"/work/my project\""
    );
    assert_eq!(WindowMatch::title("").as_str(), "title:\"\"");
    let expression = "id:7 or (title:build and not pid:123)";
    assert_eq!(WindowMatch::raw(expression).as_str(), expression);
}

#[test]
fn listing_constructors_keep_native_omission_and_self_rules() {
    assert_eq!(payload(&Ls::all()), json!({"all_env_vars": false}));
    assert_eq!(
        payload(&Ls::window(WindowId(7))),
        json!({"all_env_vars": false, "match": "id:7"})
    );
    assert_eq!(
        payload(&Ls::tab(TabMatch::title("work"))),
        json!({"all_env_vars": false, "match_tab": "title:work"})
    );
    assert_eq!(
        payload(&Ls::calling_window()),
        json!({"all_env_vars": false, "self": true})
    );
    assert_eq!(
        payload(
            &Ls::builder()
                .match_spec(WindowId(3))
                .match_tab(TabId(9))
                .build()
        ),
        json!({"all_env_vars": false, "match": "id:3", "match_tab": "id:9"})
    );
    assert_eq!(
        payload(&Ls::builder().maybe_match_spec(Some(WindowId(4))).build()),
        json!({"all_env_vars": false, "match": "id:4"})
    );
}

#[test]
fn focus_and_text_constructors_keep_distinct_payload_profiles() {
    assert_eq!(
        payload(&FocusWindow::id(WindowId(7))),
        json!({"match": "id:7"})
    );
    assert_eq!(
        payload(&GetText::new(TextExtent::Screen)),
        json!({"match": null, "extent": "screen", "ansi": false, "cursor": false,
               "wrap_markers": false, "clear_selection": false, "self": false})
    );
    assert_eq!(
        payload(&GetText::window(WindowId(8), TextExtent::Selection)),
        json!({"match": "id:8", "extent": "selection", "ansi": false, "cursor": false,
               "wrap_markers": false, "clear_selection": false, "self": false})
    );
    assert_eq!(
        payload(&GetText::calling_window(TextExtent::All)),
        json!({"match": null, "extent": "all", "ansi": false, "cursor": false,
               "wrap_markers": false, "clear_selection": false, "self": true})
    );
    assert_eq!(
        payload(&LaunchWait::new(["sh", "-c", "exit 7"])),
        json!({"args": ["sh", "-c", "exit 7"], "wait_for_child_to_exit": true, "type": "window"})
    );
}

#[test]
fn window_sets_keep_input_match_field_and_explicit_all_flag() {
    let send = |windows| SendText {
        data: "hello".into(),
        windows,
        ..SendText::default()
    };
    assert_eq!(
        payload(&send(WindowSet::window(WindowId(4)))),
        json!({"match": "id:4", "match_tab": null, "all": false, "exclude_active": false,
               "data": "text:hello", "bracketed_paste": "disable"})
    );
    assert_eq!(
        payload(&send(WindowSet::tab(TabId(9))))["match_tab"],
        "id:9"
    );
    assert_eq!(
        payload(&send(WindowSet::all())),
        json!({"match": null, "match_tab": null, "all": true, "exclude_active": false,
               "data": "text:hello", "bracketed_paste": "disable"})
    );
    assert_eq!(
        payload(&send(
            WindowSet::builder().match_window(WindowId(5)).build()
        ))["match"],
        "id:5"
    );
}

#[test]
fn window_constructors_and_builder_defaults_keep_operation_fields() {
    assert_eq!(
        payload(&WindowCommand::close(WindowId(7))),
        json!({"match": "id:7", "self": false, "ignore_no_match": false})
    );
    assert_eq!(
        payload(&WindowCommand::title(
            WindowMatch::title("work"),
            "a 'literal' title"
        )),
        json!({"match": "title:work", "self": false, "title": "a 'literal' title", "temporary": false})
    );
    assert_eq!(
        payload(&WindowCommand::reset_title(WindowTarget::CallingWindow)),
        json!({"match": null, "self": true, "temporary": false})
    );
    assert_eq!(
        payload(&WindowCommand::scroll(WindowId(2), ScrollAmount::End)),
        json!({"match": "id:2", "self": false, "amount": ["end", null]})
    );
    assert_eq!(
        payload(
            &WindowCommand::builder()
                .operation(WindowOperation::RemoveMarker)
                .build()
        ),
        json!({"match": null, "self": false})
    );
    assert_eq!(
        payload(&ResizeWindow::new(ResizeAxis::Vertical, -2)),
        json!({"match": null, "self": false, "axis": "vertical", "increment": -2})
    );
    assert_eq!(
        payload(&SelectWindow::new(TabId(3))),
        json!({"match": "id:3", "self": false, "title": null, "exclude_active": false, "reactivate_prev_tab": false})
    );
    assert_eq!(
        payload(&SendText::new("literal \\n")),
        json!({"match": null, "match_tab": null, "all": false, "exclude_active": false,
               "data": "text:literal \\n", "bracketed_paste": "disable"})
    );
    assert_eq!(
        payload(&UserVars::query(WindowId(5))),
        json!({"match": "id:5", "self": false, "var": []})
    );
}

#[test]
fn tab_constructors_preserve_layout_and_typed_detach_destinations() {
    assert_eq!(
        payload(&TabCommand::focus(TabId(8))),
        json!({"match": "id:8", "self": false})
    );
    assert_eq!(
        payload(&TabCommand::builder().operation(TabOperation::Focus).build()),
        json!({"match": null, "self": false})
    );
    assert_eq!(
        payload(&TabCommand::layout(TabMatch::title("work"), Layout::Stack)),
        json!({"match": "title:work", "self": false, "layout": "stack"})
    );
    assert_eq!(
        payload(&TabCommand::enabled_layouts(
            TabTarget::All,
            [Layout::Tall, Layout::Splits]
        )),
        json!({"match": "all", "self": false, "layouts": ["tall", "splits"], "configured": false})
    );
    assert_eq!(
        payload(&TabCommand::detach(
            TabId(3),
            TabDetachDestination::NewOsWindow
        )),
        json!({"match": "id:3", "self": false, "target_tab": null})
    );
    assert_eq!(
        payload(&TabCommand::detach(TabTarget::CallingTab, TabId(4))),
        json!({"match": null, "self": true, "target_tab": "id:4"})
    );
}

#[test]
fn typed_keys_use_native_shortcut_spelling_and_preserve_raw_extensions() {
    let command = SendKey::new([
        KeyChord::ctrl('a'),
        KeyChord::modified(Modifiers::CTRL | Modifiers::SHIFT, Key::PageUp),
        KeyChord::new('+'),
        KeyChord::alt(' '),
        KeyChord::new('é'),
        KeyChord::new(Key::Enter),
        KeyChord::raw("kitty_mod+f35"),
        KeyChord::new(Key::raw("kp_add")),
    ]);
    assert_eq!(command.request().cmd, "send-key");
    assert_eq!(
        payload(&command),
        json!({"match": null, "match_tab": null, "all": false, "exclude_active": false,
               "keys": ["ctrl+a", "ctrl+shift+page_up", "plus", "alt+space", "é", "enter", "kitty_mod+f35", "kp_add"]})
    );
    assert_eq!(
        payload(
            &SendKey::builder()
                .keys(vec![Key::Escape.into(), KeyChord::shift(Key::Tab)])
                .build()
        )["keys"],
        json!(["escape", "shift+tab"])
    );
    assert_eq!(
        KeyChord::modified(
            Modifiers::CTRL
                | Modifiers::CTRL
                | Modifiers::SHIFT
                | Modifiers::ALT
                | Modifiers::SUPER
                | Modifiers::HYPER
                | Modifiers::META
                | Modifiers::CAPS_LOCK
                | Modifiers::NUM_LOCK
                | Modifiers::KITTY_MOD,
            Key::F1,
        )
        .to_string(),
        "ctrl+shift+alt+super+hyper+meta+caps_lock+num_lock+kitty_mod+f1"
    );
}

#[test]
fn signals_are_symbolic_names_and_empty_rc_list_still_defaults_to_interrupt() {
    assert_eq!(
        payload(&WindowCommand::signal(
            WindowId(7),
            [Signal::Term, Signal::Usr1, Signal::Custom("sigpwr".into())]
        )),
        json!({"match": "id:7", "self": false, "signals": ["SIGTERM", "SIGUSR1", "SIGPWR"]})
    );
    assert_eq!(
        payload(&WindowCommand::signal(WindowTarget::CallingWindow, [])),
        json!({"match": null, "self": true, "signals": ["SIGINT"]})
    );
    assert_eq!(
        payload(&Action::new(KnownAction::SignalChild(vec![
            Signal::Hup,
            Signal::Custom("sigwinch".into())
        ]))),
        json!({"match_window": null, "self": false, "action": "signal_child SIGHUP SIGWINCH"})
    );
}

#[test]
fn simple_actions_construct_directly_and_target_with_typed_builder_values() {
    assert_eq!(
        payload(&Action::new(SimpleAction::NextTab)),
        json!({"match_window": null, "self": false, "action": "next_tab"})
    );
    assert_eq!(
        payload(
            &Action::builder()
                .action(SimpleAction::ScrollHome)
                .target(WindowId(8))
                .build()
        ),
        json!({"match_window": "id:8", "self": false, "action": "scroll_home"})
    );
}
