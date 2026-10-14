//! Golden payloads are independently specified from stable v0.49.2
//! b2cb871254fbdac19a085188813c8dfb258b95cd `message_to_kitty` constructors and
//! `response_from_kitty` handlers, not request -> response mirror round trips.
use kitty_rc_commands::actions::*;
use kitty_rc_commands::appearance::*;
use kitty_rc_commands::configuration::*;
use kitty_rc_commands::process::*;
use kitty_rc_commands::tab::*;
use kitty_rc_commands::targets::*;
use kitty_rc_commands::window::*;
use kitty_rc_commands::{
    Class, Command, DecodeError, GetText, Ls, Mode, RawFinite, SessionText, Submission, TextExtent,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn payload(command: &impl Command) -> Value {
    command.request().payload.unwrap()
}

#[test]
fn action_is_one_native_string_and_match_window_not_send_key() {
    let action = Action::builder()
        .action(KnownAction::GotoTab(2))
        .target(WindowTarget::Id(WindowId(7)))
        .build();
    assert_eq!(action.request().cmd, "action");
    assert_eq!(
        payload(&action),
        json!({"action":"goto_tab 2","match_window":"id:7","self":false})
    );
    assert!(!action.reusable());
    let raw = Action {
        action: ActionSpec::Raw("combine : next_tab : goto_layout stack".into()),
        target: WindowTarget::CallingWindow,
    };
    assert_eq!(
        payload(&raw),
        json!({"action":"combine : next_tab : goto_layout stack","match_window":null,"self":true})
    );
    assert!(!raw.reusable());
    assert!(Action::decode(None).is_ok()); // dispatch ack, not action result.
}

#[test]
fn nested_grammars_have_distinct_string_shell_and_integer_profiles() {
    // options/utils.py: remainder-string titles are NOT shell quoted.
    assert_eq!(
        KnownAction::SetWindowTitle("a 'quoted' title".into()).wire(),
        "set_window_title a 'quoted' title"
    );
    // kitten_parse/shlex_parse/load_config_file: actual argv boundaries survive
    // space, apostrophe and empty argument. Upstream to_cmdline subsequently
    // expands tilde/environment variables even inside quoted source tokens.
    assert_eq!(
        KnownAction::Kitten {
            name: "custom tool.py".into(),
            args: vec!["don't".into(), "".into(), "$HOME".into()]
        }
        .wire(),
        "kitten 'custom tool.py' 'don'\"'\"'t' '' '$HOME'"
    );
    assert_eq!(
        KnownAction::LoadConfigFiles(vec!["a b.conf".into()]).wire(),
        "load_config_file 'a b.conf'"
    );
    assert_eq!(
        KnownAction::NewProcess {
            kind: NewProcess::Launch,
            args: vec![
                "--type=tab".into(),
                "sh".into(),
                "-c".into(),
                "exit 7".into()
            ]
        }
        .wire(),
        "launch '--type=tab' 'sh' '-c' 'exit 7'"
    );
    assert_eq!(KnownAction::GotoTab(-1).wire(), "goto_tab -1"); // native parser's special negative rule is not pre-applied twice.
    assert_eq!(
        KnownAction::NeighboringWindow(Direction::Bottom).wire(),
        "neighboring_window bottom"
    );
    assert_eq!(
        KnownAction::ResizeWindow {
            quality: ResizeQuality::Narrower,
            increment: 3
        }
        .wire(),
        "resize_window narrower 3"
    );
    assert_eq!(
        KnownAction::ScrollToPrompt {
            count: -1,
            offset: 2
        }
        .wire(),
        "scroll_to_prompt -1 2"
    );
    assert_eq!(
        KnownAction::ClearTerminal {
            action: ClearTerminal::ToCursorScroll,
            active_only: false
        }
        .wire(),
        "clear_terminal to_cursor_scroll all"
    );
    assert_eq!(
        KnownAction::FontSize {
            all: false,
            change: FontChange::divide(2.0).unwrap()
        }
        .wire(),
        "change_font_size current /2"
    );
}

#[test]
fn kitten_name_args_and_actual_return_contract() {
    let kitten = Kitten::builder()
        .name("custom.py")
        .args(vec!["--flag".into(), "two words".into()])
        .target(WindowTarget::Id(WindowId(11)))
        .build();
    assert_eq!(
        payload(&kitten),
        json!({"kitten":"custom.py","args":["--flag","two words"],"match":"id:11","self":false})
    );
    assert_eq!(Kitten::decode(None).unwrap(), KittenResult::NoData);
    assert_eq!(
        Kitten::decode(Some(json!(false))).unwrap(),
        KittenResult::Boolean(false)
    );
    assert_eq!(
        Kitten::decode(Some(json!(""))).unwrap(),
        KittenResult::Text("".into())
    );
    assert!(Kitten::decode(Some(json!({"not":"a no_ui return"}))).is_err());
    assert!(Kitten::decode(Some(Value::Null)).is_err());
    assert!(!kitten.reusable());
}

#[test]
fn config_paths_and_override_are_lists_and_env_is_dictionary() {
    let load = LoadConfig::builder()
        .paths(vec!["relative.conf".into(), "/server/two.conf".into()])
        .overrides(vec!["font_size=16".into(), "background=#112233".into()])
        .ignore_previous_overrides(true)
        .build();
    assert_eq!(
        payload(&load),
        json!({"paths":["relative.conf","/server/two.conf"],"override":["font_size=16","background=#112233"],"ignore_overrides":true})
    );
    assert_eq!(
        payload(&LoadConfig::default()),
        json!({"paths":[],"override":[],"ignore_overrides":false})
    );
    let env = Env {
        changes: vec![
            Assignment::Set {
                name: "A".into(),
                value: "b=c".into(),
            },
            Assignment::Remove("OLD".into()),
        ],
    };
    assert_eq!(payload(&env), json!({"env":{"A":"b=c","OLD=":""}}));
}

#[test]
fn stable_resize_is_integer_axis_not_scroll_amount_or_head_fraction() {
    let resize = ResizeWindow::builder()
        .target(WindowTarget::CallingWindow)
        .axis(ResizeAxis::Vertical)
        .increment(-3)
        .build();
    assert_eq!(
        payload(&resize),
        json!({"match":null,"self":true,"axis":"vertical","increment":-3})
    );
    assert_eq!(ResizeWindow::decode(None).unwrap(), ResizeResult::NoData);
    assert_eq!(
        ResizeWindow::decode(Some(json!(false))).unwrap(),
        ResizeResult::Boolean(false)
    );
    // RemoteControl.handle_cmd emits ok:true with a string; the Go reference
    // client interprets string_return_is_error. Do not conflate with ok:false.
    assert_eq!(
        ResizeWindow::decode(Some(json!("This layout cannot be resized"))).unwrap(),
        ResizeResult::Rejected("This layout cannot be resized".into())
    );
    assert!(ResizeWindow::decode(Some(json!(3))).is_err());
}

#[test]
fn window_title_close_detach_and_marker_fixtures() {
    let cmd = |operation| WindowCommand {
        target: WindowTarget::Id(WindowId(8)),
        operation,
    };
    assert_eq!(
        payload(&cmd(WindowOperation::Close {
            ignore_no_match: true
        })),
        json!({"match":"id:8","self":false,"ignore_no_match":true})
    );
    assert_eq!(
        payload(&cmd(WindowOperation::Title {
            title: None,
            temporary: false
        })),
        json!({"match":"id:8","self":false,"temporary":false})
    );
    assert_eq!(
        payload(&cmd(WindowOperation::Detach {
            destination: DetachDestination::NewTab,
            stay_in_tab: true
        })),
        json!({"match":"id:8","self":false,"target_tab":"new","stay_in_tab":true})
    );
    assert_eq!(
        payload(&cmd(WindowOperation::Detach {
            destination: DetachDestination::Tab(TabId(12)),
            stay_in_tab: false
        }))["target_tab"],
        "id:12"
    );
    assert_eq!(
        payload(&cmd(WindowOperation::RemoveLogo)),
        json!({"match":"id:8","self":false,"data":"-","alpha":-1,"position":""})
    );
    let spec = MarkerSpec::Patterns {
        kind: MarkerKind::Itext,
        first: (MarkerGroup::Two, "ERROR text".into()),
        additional: vec![],
    };
    assert_eq!(
        payload(&cmd(WindowOperation::CreateMarker(spec.clone())))["marker_spec"],
        json!(["itext", "2", "ERROR text"])
    );
    assert_eq!(
        KnownAction::ToggleMarker(spec).wire(),
        "toggle_marker itext '2' 'ERROR text'"
    );
    assert!(
        !cmd(WindowOperation::CreateMarker(MarkerSpec::Function(
            "server_extension.py".into()
        )))
        .reusable()
    );
}

#[test]
fn scroll_amount_pair_and_typed_integral_units() {
    let scroll = |amount| WindowCommand {
        target: WindowTarget::CallingWindow,
        operation: WindowOperation::Scroll(amount),
    };
    assert_eq!(
        payload(&scroll(ScrollAmount::Pages(Finite::new(-0.5).unwrap()))),
        json!({"match":null,"self":true,"amount":[-0.5,"p"]})
    );
    assert_eq!(
        payload(&scroll(ScrollAmount::Start))["amount"],
        json!(["start", null])
    );
    assert_eq!(
        payload(&scroll(ScrollAmount::Unscroll(3)))["amount"],
        json!([3, "u"])
    );
    assert_eq!(
        payload(&scroll(ScrollAmount::Prompts(-2)))["amount"],
        json!([-2, "r"])
    );
}

#[test]
fn select_window_is_deferred_required_integer_not_absent_or_string() {
    let select = SelectWindow::builder()
        .target(TabTarget::Id(TabId(3)))
        .title("pick one")
        .exclude_active(true)
        .reactivate_previous_tab(true)
        .build();
    assert_eq!(
        payload(&select),
        json!({"match":"id:3","self":false,"title":"pick one","exclude_active":true,"reactivate_prev_tab":true})
    );
    assert_eq!(select.mode(), Mode::Deferred);
    assert_eq!(select.class(), Class::LongWait);
    assert_eq!(SelectWindow::decode(Some(json!(9))).unwrap(), WindowId(9));
    for bad in [None, Some(json!("9")), Some(json!(0)), Some(json!(-1))] {
        assert!(SelectWindow::decode(bad).is_err());
    }
}

#[test]
fn tab_layout_and_colors_follow_handler_fields() {
    let command = TabCommand {
        target: TabTarget::All,
        operation: TabOperation::EnabledLayouts {
            layouts: vec![Layout::Custom("tall:bias=70".into()), Layout::Stack],
            configured: true,
        },
    };
    assert_eq!(
        payload(&command),
        json!({"match":"all","self":false,"layouts":["tall:bias=70","stack"],"configured":true})
    );
    let colors = TabCommand {
        target: TabTarget::CallingTab,
        operation: TabOperation::Colors(BTreeMap::from([
            (TabColor::ActiveFg, Some(Rgb::new(0x112233).unwrap())),
            (TabColor::InactiveBg, None),
        ])),
    };
    assert_eq!(
        payload(&colors),
        json!({"match":null,"self":true,"colors":{"active_fg":1122867,"inactive_bg":null}})
    );
    let last = TabCommand {
        target: TabTarget::All,
        operation: TabOperation::LastUsedLayout,
    };
    assert_eq!(
        payload(&last),
        json!({"match":"all","self":false,"all":true})
    );
}

#[test]
fn input_submissions_retire_and_preserve_literal_utf8() {
    let keys = SendKey::builder()
        .keys(vec!["ctrl+a".into(), "ctrl+b".into()])
        .exclude_active(true)
        .build();
    assert_eq!(
        payload(&keys),
        json!({"keys":["ctrl+a","ctrl+b"],"match":null,"match_tab":null,"all":false,"exclude_active":true})
    );
    let text = SendText::builder()
        .text("\\e[31m 🌊")
        .bracketed_paste(BracketedPaste::Auto)
        .build();
    assert_eq!(payload(&text)["data"], "text:\\e[31m 🌊"); // literal backslash, not CLI Python escape parsing
    assert_eq!(payload(&text)["bracketed_paste"], "auto");
    for (mode, reusable) in [
        (keys.mode(), keys.reusable()),
        (text.mode(), text.reusable()),
    ] {
        assert_eq!(mode, Mode::SubmissionOnly);
        assert!(!reusable);
    }
    let submission = Submission(LoadConfig::default());
    assert_eq!(submission.mode(), Mode::SubmissionOnly);
    assert!(!submission.reusable());
}

#[test]
fn user_vars_keep_list_wire_and_required_string_result() {
    let vars = UserVars::builder()
        .target(WindowTarget::CallingWindow)
        .operations(vec![
            Assignment::Set {
                name: "project".into(),
                value: "a=b".into(),
            },
            Assignment::Remove("old".into()),
        ])
        .build();
    assert_eq!(
        payload(&vars),
        json!({"match":null,"self":true,"var":["project=a=b","old"]})
    );
    assert_eq!(UserVars::decode(Some(json!(""))).unwrap(), "");
    assert!(UserVars::decode(None).is_err());
}

#[test]
fn font_opacity_rgb_and_no_nan_null_coercion() {
    assert!(Finite::new(f64::NAN).is_err());
    assert!(Finite::new(f64::INFINITY).is_err());
    assert!(Opacity::new(-0.1).is_err());
    assert!(Opacity::new(1.1).is_err());
    assert!(Rgb::new(0x1000000).is_err());
    assert!(FontChange::divide(0.0).is_err());
    assert!(FontChange::absolute(-1.0).is_err());
    let font = SetFontSize::builder()
        .change(FontChange::multiply(1.5).unwrap())
        .all(true)
        .build();
    assert_eq!(
        payload(&font),
        json!({"size":1.5,"all":true,"increment_op":"*"})
    );
    assert_eq!(
        payload(&SetFontSize {
            change: FontChange::reset(),
            all: false
        }),
        json!({"size":0.0,"all":false,"increment_op":null})
    );
    let opacity = Appearance {
        windows: WindowSet {
            match_window: Some("id:4".into()),
            match_tab: None,
            all: false,
        },
        operation: AppearanceOperation::Opacity {
            opacity: Opacity::new(0.4).unwrap(),
            toggle: true,
        },
    };
    assert_eq!(
        payload(&opacity),
        json!({"match_window":"id:4","match_tab":null,"all":false,"opacity":0.4,"toggle":true})
    );
}

#[test]
fn appearance_reset_spacing_and_finite_background_special_values() {
    let command = |operation| Appearance {
        windows: WindowSet::default(),
        operation,
    };
    assert_eq!(
        payload(&command(AppearanceOperation::ResetColors)),
        json!({"match_window":null,"match_tab":null,"all":true,"configured":true,"colors":{},"reset":true})
    );
    let spacing = command(AppearanceOperation::Spacing {
        settings: BTreeMap::from([
            (SpacingEdge::MarginTop, Some(Finite::new(10.5).unwrap())),
            (SpacingEdge::PaddingLeft, None),
        ]),
        configured: true,
    });
    assert_eq!(
        payload(&spacing)["settings"],
        json!({"margin-top":10.5,"padding-left":null})
    );
    let image = command(AppearanceOperation::BackgroundImage {
        image: BackgroundImage::Increment(0),
        layout: BackgroundLayout::CenteredScaled,
    });
    assert_eq!(
        payload(&image),
        json!({"match":null,"match_tab":null,"all":false,"configured":false,"data":"index:+0","layout":"cscaled"})
    );
    assert_eq!(
        payload(&command(AppearanceOperation::BackgroundImage {
            image: BackgroundImage::Remove { configured: true },
            layout: BackgroundLayout::Configured
        }))["data"],
        "-"
    );
    assert!(image.request().stream.is_none());
    assert!(image.request().stream_id.is_none());
}

#[test]
fn launch_args_env_copy_env_and_window_id_string_contract() {
    let launch = Launch::builder()
        .kind(LaunchType::Tab)
        .args(vec!["sh".into(), "-c".into(), "printf x".into()])
        .env(vec![Assignment::Set {
            name: "A".into(),
            value: "b=c".into(),
        }])
        .copy_env(vec!["LOCAL=value".into()])
        .location(Location::Before)
        .keep_focus(true)
        .build();
    let p = payload(&launch);
    assert_eq!(p["type"], "tab");
    assert_eq!(p["args"], json!(["sh", "-c", "printf x"]));
    assert_eq!(p["env"], json!(["A=b=c"]));
    assert_eq!(p["copy_env"], json!(["LOCAL=value"]));
    assert_eq!(p["location"], "before");
    assert_eq!(p["wait_for_child_to_exit"], false);
    assert!(p.get("window_type").is_none());
    assert_eq!(
        Launch::decode(Some(json!("12"))).unwrap(),
        LaunchResult::Window(WindowId(12))
    );
    assert_eq!(
        Launch::decode(Some(json!("0"))).unwrap(),
        LaunchResult::NoWindow
    );
    for bad in [
        None,
        Some(json!(12)),
        Some(json!("-1")),
        Some(json!("SIGTERM")),
        Some(json!("")),
    ] {
        assert!(Launch::decode(bad).is_err());
    }
}

#[test]
fn buffered_run_result_is_binary_and_nonzero_exit_is_not_rc_failure() {
    // run.py on_death sends all four fields as object; stdout/stderr are
    // standard base64, not UTF-8 strings or streaming progress chunks.
    let output = RunOutput::decode(Some(
        json!({"stdout":"AP8=","stderr":"ZXJyb3IK","exit_code":7,"exit_status":1792,"future":true}),
    ))
    .unwrap();
    assert_eq!(output.stdout, [0, 255]);
    assert_eq!(output.stderr, b"error\n");
    assert_eq!(output.exit_code, 7);
    assert!(RunOutput::decode(Some(json!({"stdout":"","stderr":"","exit_code":0}))).is_err());
    assert!(
        RunOutput::decode(Some(
            json!({"stdout":"%%%","stderr":"","exit_code":0,"exit_status":0})
        ))
        .is_err()
    );
    assert!(RunOutput::decode(None).is_err());
}

#[test]
fn screenshot_data_and_server_save_are_distinct_deferred_profiles() {
    assert!(ServerPath::new("").is_err());
    assert!(ServerPath::new("bad\0path").is_err());
    let save = SaveScreenshot::builder()
        .server_path(ServerPath::new("~/capture.png").unwrap())
        .build();
    assert_eq!(
        payload(&save),
        json!({"match":null,"match_tab":null,"output_path":"~/capture.png"})
    );
    let shot = Screenshot::builder()
        .target(ScreenshotTarget::Window("id:8".into()))
        .build();
    assert_eq!(
        payload(&shot),
        json!({"match":"id:8","match_tab":null,"output_path":""})
    );
    assert_eq!(shot.mode(), Mode::Deferred);
    assert_eq!(
        Screenshot::decode(Some(json!("iVBORw0KGgo="))).unwrap(),
        b"\x89PNG\r\n\x1a\n"
    ); // validates signature, not complete image decoding
    assert!(Screenshot::decode(Some(json!("bm90IGEgcG5n"))).is_err());
    assert!(Screenshot::decode(Some(json!("Screenshot saved to: /server/a.png"))).is_err());
    assert_eq!(
        SaveScreenshot::decode(Some(json!("Screenshot saved to: /server/a.png"))).unwrap(),
        "/server/a.png"
    );
    assert!(SaveScreenshot::decode(Some(json!("iVBORw0KGgo="))).is_err());
    assert!(SaveScreenshot::decode(None).is_err());
}

#[test]
fn listing_required_identity_and_containers_do_not_become_empty_successes() {
    for bad in [
        json!([{}]),
        json!([{"id":1}]),
        json!([{"id":1,"tabs":[{"id":2}]}]),
        json!([{"id":1,"tabs":[{"id":2,"windows":[{}]}]}]),
        json!([{"id":null,"tabs":[]}]),
    ] {
        assert!(Ls::decode(Some(json!(bad.to_string()))).is_err(), "{bad}");
    }
    let good = json!([{"id":1,"tabs":[{"id":2,"windows":[{"id":3,"future":{"x":1}}],"future":true}],"future":true}]);
    assert_eq!(
        Ls::decode(Some(json!(good.to_string()))).unwrap()[0].tabs[0].windows[0].id,
        Some(3)
    );
}

#[test]
fn extended_text_and_session_string_output_profiles() {
    let text = GetText::builder()
        .extent(TextExtent::AlternateScrollback)
        .ansi(true)
        .cursor(true)
        .clear_selection(true)
        .only_self(true)
        .build();
    assert_eq!(
        payload(&text),
        json!({"match":null,"extent":"alternate_scrollback","ansi":true,"cursor":true,"wrap_markers":false,"clear_selection":true,"self":true})
    );
    assert_eq!(
        payload(&SessionText),
        json!({"output_format":"session","all_env_vars":false})
    );
    assert!(matches!(
        SessionText::decode(None),
        Err(DecodeError::MissingData)
    ));
}

#[test]
fn raw_defaults_retire_and_catalog_has_all_stable_names_plus_explicit_head_only() {
    use kitty_rc_commands::catalog::{CAPABILITIES, Coverage, STABLE_REVISION, capability};
    assert_eq!(
        CAPABILITIES
            .iter()
            .filter(|c| c.source_revision == STABLE_REVISION)
            .count(),
        41
    );
    let names: std::collections::BTreeSet<_> = CAPABILITIES.iter().map(|c| c.name).collect();
    assert_eq!(names.len(), CAPABILITIES.len());
    assert_eq!(capability("run").unwrap().coverage, Coverage::TypedUpload);
    assert_eq!(
        capability("new-window").unwrap().coverage,
        Coverage::DeprecatedRaw
    );
    assert_eq!(
        capability("resize-window-edge").unwrap().coverage,
        Coverage::HeadOnlyRaw
    );
    assert_eq!(
        capability("screenshot").unwrap().earliest_documented,
        Some([0, 49, 0])
    );
    assert!(capability("action").unwrap().earliest_documented.is_none());
    assert!(
        !RawFinite {
            name: "custom".into(),
            payload: json!({}),
            class: Class::Short,
            submission_only: false
        }
        .reusable()
    );
}
