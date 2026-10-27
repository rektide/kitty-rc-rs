//! Independently specified native payloads from kitty v0.49.2: rc/set_colors.py,
//! rc/set_window_logo.py, rc/load_config.py, rc/screenshot.py, launch.py and
//! options/definition.py. These assert wire defaults and grammar, rather than
//! treating agreement between our builders and constructors as evidence.
use kitty_rc_commands::Command;
use kitty_rc_commands::appearance::{
    Appearance, AppearanceOperation, BackgroundImage, ColorName, ColorValue, FontChange, GetColors, LigatureStrategy, LogoPosition, SetFontSize, SpacingEdge,
};
use kitty_rc_commands::configuration::{ConfigOverride, Env, Kitten, LoadConfig};
use kitty_rc_commands::process::{Launch, LaunchCwd, SaveScreenshot, Screenshot, ScreenshotTarget};
use kitty_rc_commands::targets::{Assignment, Finite, Opacity, Rgb, ServerPath, TabId, TabMatch, WindowId, WindowMatch, WindowSet, WindowTarget};
use kitty_rc_commands::upload::{BackgroundPng, LogoPng, Run};
use serde_json::{Value, json};

fn payload(command: &impl Command) -> Value {
    command.request().payload.unwrap()
}

#[test]
fn appearance_builder_and_static_constructors_preserve_native_scopes() {
    let ligatures = Appearance::builder()
        .operation(AppearanceOperation::Ligatures(LigatureStrategy::Cursor))
        .build();
    assert_eq!(ligatures.request().cmd, "disable-ligatures");
    assert_eq!(
        payload(&ligatures),
        json!({"match_window":null,"match_tab":null,"all":false,"strategy":"cursor"})
    );
    assert_eq!(
        payload(&Appearance::ligatures(WindowSet::tab(TabMatch::id(TabId(3))), LigatureStrategy::Always,)),
        json!({"match_window":null,"match_tab":"id:3","all":false,"strategy":"always"})
    );
    assert_eq!(
        payload(&Appearance::opacity(
            WindowSet::window(WindowMatch::id(WindowId(7))),
            Opacity::new(0.25).unwrap(),
        )),
        json!({"match_window":"id:7","match_tab":null,"all":false,"opacity":0.25,"toggle":false})
    );
    assert_eq!(
        payload(&Appearance::spacing(
            WindowSet::default(),
            [(SpacingEdge::PaddingTop, Some(Finite::new(2.5).unwrap())), (SpacingEdge::MarginLeft, None),],
        )),
        json!({"match_window":null,"match_tab":null,"all":false,"configured":false,
            "settings":{"padding-top":2.5,"margin-left":null}})
    );
    // Native reset always acts globally and updates configured colors.
    assert_eq!(
        payload(&Appearance::reset_colors()),
        json!({"match_window":null,"match_tab":null,"all":true,"colors":{},"reset":true,"configured":true})
    );
    assert_eq!(
        payload(&Appearance::background_image(
            WindowSet::window(WindowMatch::id(WindowId(9))),
            BackgroundImage::Increment(0),
        )),
        json!({"match":"id:9","match_tab":null,"all":false,"configured":false,
            "data":"index:+0","layout":"configured"})
    );
}

#[test]
fn typed_color_names_serialize_as_native_dictionary_keys() {
    let colors = Appearance::colors(
        WindowSet::default(),
        [
            (ColorName::Foreground, ColorValue::Rgb(Rgb::new(0x123456).unwrap())),
            (ColorName::Background, ColorValue::Rgb(Rgb::new(0).unwrap())),
            (ColorName::CursorTextColor, ColorValue::Reset),
            (ColorName::SelectionForeground, ColorValue::Reset),
            (ColorName::Palette(0), ColorValue::Rgb(Rgb::new(0xffffff).unwrap())),
            (ColorName::Palette(255), ColorValue::Reset),
            (ColorName::TransparentBackgroundColors, ColorValue::TransparentBackgrounds("#112233@0.5".into())),
            (ColorName::Custom("future_color".into()), ColorValue::Reset),
        ],
    );
    assert_eq!(colors.request().cmd, "set-colors");
    assert_eq!(
        payload(&colors),
        json!({"match_window":null,"match_tab":null,"all":false,"configured":false,"reset":false,
            "colors":{"foreground":1193046,"background":0,"cursor_text_color":null,
                "selection_foreground":null,"color0":16777215,"color255":null,
                "transparent_background_colors":"#112233@0.5","future_color":null}})
    );
    assert_eq!(payload(&GetColors::new()), json!({"match":null,"configured":false}));
    assert_eq!(payload(&GetColors::for_window(WindowId(4))), json!({"match":"id:4","configured":false}));
    assert_eq!(
        payload(&GetColors::builder().match_spec(WindowMatch::title("^editor$")).configured(true).build()),
        json!({"match":"title:^editor$","configured":true})
    );
}

#[test]
fn all_logo_positions_have_exact_native_names_including_configured_empty() {
    for (position, native) in [
        (LogoPosition::Configured, ""),
        (LogoPosition::TopLeft, "top-left"),
        (LogoPosition::Top, "top"),
        (LogoPosition::TopRight, "top-right"),
        (LogoPosition::Left, "left"),
        (LogoPosition::Center, "center"),
        (LogoPosition::Right, "right"),
        (LogoPosition::BottomLeft, "bottom-left"),
        (LogoPosition::Bottom, "bottom"),
        (LogoPosition::BottomRight, "bottom-right"),
    ] {
        let logo = LogoPng::builder().position(position).build();
        assert_eq!(payload(&logo), json!({"match":null,"self":false,"position":native,"alpha":-1.0}));
        let launch = Launch::builder().logo_position(position).build();
        assert_eq!(payload(&launch)["logo_position"], native);
    }
    assert_eq!(
        payload(&LogoPng::new(WindowTarget::CallingWindow)),
        json!({"match":null,"self":true,"position":"","alpha":-1.0})
    );
    assert_eq!(
        payload(
            &LogoPng::builder()
                .target(WindowTarget::Id(WindowId(11)))
                .position(LogoPosition::BottomRight)
                .alpha(Opacity::new(0.0).unwrap())
                .build()
        ),
        json!({"match":"id:11","self":false,"position":"bottom-right","alpha":0.0})
    );
}

#[test]
fn launch_cwd_is_typed_and_matches_remain_native_strings() {
    for (cwd, native) in [
        (LaunchCwd::Current, "current"),
        (LaunchCwd::LastReported, "last_reported"),
        (LaunchCwd::Oldest, "oldest"),
        (LaunchCwd::Root, "root"),
        (LaunchCwd::Path(ServerPath::new("/server/a b").unwrap()), "/server/a b"),
        (LaunchCwd::raw("future_selector"), "future_selector"),
    ] {
        let launch = Launch::builder()
            .cwd(cwd)
            .next_to(WindowMatch::id(WindowId(3)))
            .source_window(WindowMatch::title("^source$"))
            .build();
        let p = payload(&launch);
        assert_eq!(p["cwd"], native);
        assert_eq!(p["next_to"], "id:3");
        assert_eq!(p["source_window"], "title:^source$");
    }
    // New constructors keep native argv boundaries, absent optional settings,
    // and launch's window/active-tab defaults.
    let p = payload(&Launch::new(["printf", "%s", "", "two words", "a=b"]));
    assert_eq!(p["args"], json!(["printf", "%s", "", "two words", "a=b"]));
    assert_eq!(p["type"], "window");
    assert_eq!(p["match"], Value::Null);
    assert_eq!(p["self"], false);
    assert_eq!(p["cwd"], Value::Null);
    assert_eq!(p["next_to"], Value::Null);
    assert_eq!(p["source_window"], Value::Null);
    assert_eq!(p["logo_position"], Value::Null);
    assert_eq!(p["wait_for_child_to_exit"], false);
    assert_eq!(p["location"], "default");
}

#[test]
fn config_overrides_keep_values_separate_and_are_not_env_removals() {
    let load = LoadConfig::builder()
        .overrides(vec![
            ConfigOverride::font_size(16.5).unwrap(),
            ConfigOverride::background(Rgb::new(0x00010a).unwrap()),
            ConfigOverride::foreground(Rgb::new(0xffaacc).unwrap()),
            ConfigOverride::new("font_family", "Some Font"),
            ConfigOverride::new("env", "A=b=c"),
            ConfigOverride::new("symbol_map", ""),
            ConfigOverride::raw("future_option=unparsed"),
        ])
        .ignore_previous_overrides(true)
        .build();
    assert_eq!(load.request().cmd, "load-config");
    assert_eq!(
        payload(&load),
        json!({"paths":[],"override":["font_size=16.5","background=#00010a",
            "foreground=#ffaacc","font_family=Some Font","env=A=b=c","symbol_map=",
            "future_option=unparsed"],"ignore_overrides":true})
    );
    assert_eq!(
        payload(&LoadConfig::new(["relative.conf", "/server/a b.conf"])),
        json!({"paths":["relative.conf","/server/a b.conf"],"override":[],"ignore_overrides":false})
    );
    // The server, not this command constructor, clamps small finite sizes.
    assert_eq!(ConfigOverride::font_size(2.0).unwrap().as_argument(), "font_size=2");
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(ConfigOverride::font_size(bad).is_err());
    }
    assert_eq!(
        payload(&Env::new([
            Assignment::Set {
                name: "A".into(),
                value: "b=c".into()
            },
            Assignment::Remove("OLD".into()),
        ])),
        json!({"env":{"A":"b=c","OLD=":""}})
    );
}

#[test]
fn typed_screenshot_targets_distinguish_windows_tabs_and_server_saves() {
    assert_eq!(
        payload(&Screenshot::new(ScreenshotTarget::window_id(WindowId(8)))),
        json!({"match":"id:8","match_tab":null,"output_path":""})
    );
    assert_eq!(
        payload(&Screenshot::new(ScreenshotTarget::tab_id(TabId(8)))),
        json!({"match":null,"match_tab":"id:8","output_path":""})
    );
    assert_eq!(
        payload(&Screenshot::new(ScreenshotTarget::Tab(TabMatch::title("^work$")))),
        json!({"match":null,"match_tab":"title:^work$","output_path":""})
    );
    assert_eq!(
        payload(&SaveScreenshot::new(ServerPath::new("~/capture.png").unwrap())),
        json!({"match":null,"match_tab":null,"output_path":"~/capture.png"})
    );
    assert!(ServerPath::new("").is_err());
    assert!(ServerPath::new("bad\0path").is_err());
}

#[test]
fn upload_metadata_and_iterable_command_constructors_preserve_defaults() {
    assert_eq!(
        payload(&BackgroundPng::new()),
        json!({"match":null,"all":false,"configured":false,"layout":"configured"})
    );
    assert_eq!(
        payload(&BackgroundPng::for_window(WindowId(2))),
        json!({"match":"id:2","all":false,"configured":false,"layout":"configured"})
    );
    assert_eq!(
        payload(&Run::new(["sh", "-c", "printf '%s' 'two words'", ""])),
        json!({"cmdline":["sh","-c","printf '%s' 'two words'",""],
            "env":[],"allow_remote_control":false,"remote_control_password":[]})
    );
    assert_eq!(
        payload(&Kitten::new("custom tool.py", ["--flag", "two words", ""])),
        json!({"kitten":"custom tool.py","args":["--flag","two words",""],"match":null,"self":false})
    );
    assert_eq!(
        payload(&SetFontSize::new(FontChange::reset())),
        json!({"size":0.0,"all":false,"increment_op":null})
    );
    assert_eq!(
        payload(&SetFontSize::new(FontChange::increase(1.5).unwrap())),
        json!({"size":1.5,"all":false,"increment_op":"+"})
    );
    assert!(FontChange::divide(0.0).is_err());
    assert!(FontChange::absolute(f64::NAN).is_err());
    assert!(Opacity::new(1.01).is_err());
    assert!(Rgb::new(0x1000000).is_err());
}
