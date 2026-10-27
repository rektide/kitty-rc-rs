//! Golden requests from the audited kitty v0.49.2 RC handlers and matching
//! grammar (kitty/rc/base.py and kitty/search_query_parser.py).
use kitty_rc_commands::targets::{TabId, TabMatch, WindowId, WindowMatch, WindowSet};
use kitty_rc_commands::window::SendText;
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
    assert_eq!(WindowMatch::title(r#"^(work|build) "x"\d+$"#).as_str(), r#"title:"^(work|build) \"x\"\\d+$""#);
    assert_eq!(TabMatch::cwd("/work/my project").as_str(), "cwd:\"/work/my project\"");
    assert_eq!(WindowMatch::title("").as_str(), "title:\"\"");
    let expression = "id:7 or (title:build and not pid:123)";
    assert_eq!(WindowMatch::raw(expression).as_str(), expression);
}

#[test]
fn listing_constructors_keep_native_omission_and_self_rules() {
    assert_eq!(payload(&Ls::all()), json!({"all_env_vars": false}));
    assert_eq!(payload(&Ls::window(WindowId(7))), json!({"all_env_vars": false, "match": "id:7"}));
    assert_eq!(
        payload(&Ls::tab(TabMatch::title("work"))),
        json!({"all_env_vars": false, "match_tab": "title:work"})
    );
    assert_eq!(payload(&Ls::calling_window()), json!({"all_env_vars": false, "self": true}));
    assert_eq!(
        payload(&Ls::builder().match_spec(WindowId(3)).match_tab(TabId(9)).build()),
        json!({"all_env_vars": false, "match": "id:3", "match_tab": "id:9"})
    );
    assert_eq!(
        payload(&Ls::builder().maybe_match_spec(Some(WindowId(4))).build()),
        json!({"all_env_vars": false, "match": "id:4"})
    );
}

#[test]
fn focus_and_text_constructors_keep_distinct_payload_profiles() {
    assert_eq!(payload(&FocusWindow::id(WindowId(7))), json!({"match": "id:7"}));
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
        text: "hello".into(),
        windows,
        ..SendText::default()
    };
    assert_eq!(
        payload(&send(WindowSet::window(WindowId(4)))),
        json!({"match": "id:4", "match_tab": null, "all": false, "exclude_active": false,
               "data": "text:hello", "bracketed_paste": "disable"})
    );
    assert_eq!(payload(&send(WindowSet::tab(TabId(9))))["match_tab"], "id:9");
    assert_eq!(
        payload(&send(WindowSet::all())),
        json!({"match": null, "match_tab": null, "all": true, "exclude_active": false,
               "data": "text:hello", "bracketed_paste": "disable"})
    );
    assert_eq!(payload(&send(WindowSet::builder().match_window(WindowId(5)).build()))["match"], "id:5");
}
