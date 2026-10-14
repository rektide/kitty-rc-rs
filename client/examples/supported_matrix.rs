//! Opt-in only: invoked by kitty-rc-verify (tool/verification) inside its private fixture.
//! Exact profile evidence, not a whole-catalog/minimum-version promise.
use kitty_rc::{commands::*, *};
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, path::PathBuf, time::Duration};
use tokio::time::Instant;

type Result<T = Value> = std::result::Result<T, Box<dyn Error>>;

fn options() -> SessionOptions {
    SessionOptions {
        deadline: Instant::now() + Duration::from_secs(5),
        max_request: 256 * 1024,
        max_response: 2 * 1024 * 1024,
    }
}
fn managed_options() -> Options {
    Options {
        deadline: options().deadline,
        max_response: 1024 * 1024,
    }
}
async fn execute<C: Command>(session: &mut Session, command: C) -> Result<C::Output> {
    match session.execute_with_options(command, options()).await? {
        Completion::Reply(value) => Ok(value),
        Completion::Submitted => Err("unexpected submission-only completion".into()),
    }
}
fn ensure(condition: bool, reason: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(reason.into())
    }
}
fn record(name: &str, profile: Value, status: &str, elapsed: Duration, detail: Value) {
    println!(
        "{}",
        json!({"case":name,"profile":profile,"status":status,"elapsed_ms":elapsed.as_millis(),"detail":detail})
    );
}
fn has_color(text: &str, name: &str, color: &str) -> bool {
    text.lines().any(|line| {
        let mut words = line.split_whitespace();
        words.next() == Some(name) && words.next() == Some(color)
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure(
        args.len() == 3,
        "isolated fixture only: SOCKET KEY PASSWORD",
    )?;
    let socket = PathBuf::from(&args[0]);
    let fixture = socket.parent().ok_or("fixture has no parent")?;
    ensure(
        socket.file_name().is_some_and(|s| s == "rc.sock")
            && fixture.file_name().is_some_and(|s| {
                let name = s.to_string_lossy();
                name.starts_with("kitty-rc-fixture-")
                    || name.starts_with("kitty-rc-live-")
                    || name.starts_with("kitty-private-upload-")
            }),
        "refusing a non-conformance endpoint",
    )?;
    let version_text = std::env::var("KITTY_RC_MATRIX_VERSION")?;
    let numbers = version_text
        .split('.')
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let version: [u32; 3] = numbers.try_into().map_err(|_| "expected release version")?;
    let mode = std::env::var("KITTY_RC_MATRIX_PROTECTION")?;
    let marker = std::env::var("KITTY_RC_MATRIX_MARKER")?;
    let protection = match mode.as_str() {
        "plain" => Protection::Plain,
        "encrypted" => Protection::Encrypted {
            key: parse_server_public_key(&args[1])?,
            password: args[2].clone(),
            key_origin: "only private fixture child's published key".into(),
        },
        _ => return Err("invalid private protection profile".into()),
    };
    let endpoint = Endpoint {
        address: Address::Unix(socket.clone()),
        protection,
    };
    let mut session = Session::new(endpoint.clone());
    let windows = execute(&mut session, Ls::default()).await?;
    let id = windows
        .iter()
        .flat_map(|os| &os.tabs)
        .flat_map(|tab| &tab.windows)
        .find_map(|w| w.id)
        .ok_or("no private window")?;
    let target = targets::WindowTarget::Id(targets::WindowId(id));
    let spec = format!("id:{id}");
    let mut failures = 0;
    macro_rules! case {
        ($name:literal, $profile:expr, $body:expr) => {{
            let started = Instant::now();
            let result: Result = $body.await;
            match result {
                Ok(detail) => record($name, $profile, "pass", started.elapsed(), detail),
                Err(error) => {
                    failures += 1;
                    record($name, $profile, "fail", started.elapsed(), json!({"error":error.to_string()}));
                }
            }
        }};
    }
    record(
        "private-fixture",
        json!({"version":version_text,"protection":mode,"envelope":[0,43,1],"stable_audit":catalog::STABLE_REVISION,"direct_options":{"deadline_seconds":5,"max_request":256*1024,"max_response":2*1024*1024},"managed_options":{"deadline_seconds":5,"max_response":1024*1024}}),
        "pass",
        Duration::ZERO,
        json!({"windows":windows.iter().flat_map(|o| &o.tabs).map(|t|t.windows.len()).sum::<usize>()}),
    );
    case!(
        "direct-reuse",
        json!({"command":"ls","repetitions":5}),
        async {
            let token = session.connection_token().ok_or("no first transport")?;
            for _ in 0..5 {
                execute(&mut session, Ls::default()).await?;
                ensure(
                    session.connection_token() == Some(token),
                    "direct transport replaced",
                )?;
            }
            Ok(json!({"same_connection":true,"token":format!("{token:?}")}))
        }
    );
    case!(
        "managed-reuse-and-independent-short-work",
        json!({"commands":["ls","launch(wait)","get-text(screen)","focus-window"],"child":"sentinel; sleep 1.5; exit 7"}),
        async {
            let registry = Registry::new(Limits::default())?;
            let ep = registry.register(endpoint.clone())?;
            let driver = registry.start()?;
            let mut token = None;
            for _ in 0..5 {
                let op = registry
                    .submit(&registry.reserve(), ep, Ls::default(), managed_options())
                    .map_err(|e| e.reason)?;
                op.wait().await?;
                op.take()?;
                op.release()?;
                ensure(
                    registry.pool_stats().idle == 1,
                    "expected one managed idle transport",
                )?;
                let next = registry.pool_stats().idle_tokens[&ep][0];
                ensure(
                    token.is_none_or(|previous| previous == next),
                    "managed transport replaced",
                )?;
                token = Some(next);
            }
            let sentinel = fixture.join("long-started");
            // The Rust fixture child (kitty-rc-verify-child) owns the
            // long-running journey: write the sentinel, sleep, exit 7. No
            // embedded interpreter string remains in this example.
            let child = std::env::var("KITTY_RC_MATRIX_CHILD")?;
            let long = registry
                .submit(
                    &registry.reserve(),
                    ep,
                    LaunchWait {
                        args: vec![
                            child,
                            "sentinel".into(),
                            sentinel.to_string_lossy().into_owned(),
                            "--sleep-ms".into(),
                            "1500".into(),
                            "--exit".into(),
                            "7".into(),
                        ],
                    },
                    managed_options(),
                )
                .map_err(|e| e.reason)?;
            let deadline = Instant::now() + Duration::from_secs(3);
            while !sentinel.exists() {
                ensure(Instant::now() < deadline, "long child did not start")?;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            ensure(
                registry
                    .summary(long.token())
                    .is_some_and(|s| s.status == Status::Running),
                "long wait not running",
            )?;
            let started = Instant::now();
            let focus = registry
                .submit(
                    &registry.reserve(),
                    ep,
                    FocusWindow {
                        match_spec: spec.clone(),
                    },
                    managed_options(),
                )
                .map_err(|e| e.reason)?;
            let text = registry
                .submit(
                    &registry.reserve(),
                    ep,
                    GetText {
                        match_spec: Some(spec.clone()),
                        ..GetText::default()
                    },
                    managed_options(),
                )
                .map_err(|e| e.reason)?;
            focus.wait().await?;
            focus.take()?;
            text.wait().await?;
            let Completion::Reply(snapshot) = text.take()? else {
                return Err("missing text reply".into());
            };
            ensure(
                snapshot.contains(&marker),
                "snapshot missed private child marker",
            )?;
            let short_ms = started.elapsed().as_millis();
            let long_summary = registry.summary(long.token()).ok_or("lost long summary")?;
            ensure(
                long_summary.status == Status::Running,
                "short work waited for long completion",
            )?;
            long.wait().await?;
            ensure(
                matches!(long.take()?,Completion::Reply(code) if code == "7"),
                "wrong child exit data",
            )?;
            long.release()?;
            focus.release()?;
            text.release()?;
            let idle = registry.pool_stats().idle;
            registry.shutdown(Shutdown::Drain).await?;
            driver.stop().await;
            Ok(
                json!({"same_managed_connection_for_five_listings":true,"short_elapsed_ms":short_ms,"long_pending_after_short":true,"long_progress":format!("{:?}",long_summary.progress),"idle_after_three_concurrent_leases":idle,"exit":"7"}),
            )
        }
    );
    case!(
        "remote-rejection",
        json!({"command":"focus-window","match":"id:18446744073709551615"}),
        async {
            let token = session.connection_token();
            let failure = session
                .execute_with_options(
                    FocusWindow {
                        match_spec: "id:18446744073709551615".into(),
                    },
                    options(),
                )
                .await
                .err()
                .ok_or("no-match unexpectedly succeeded")?;
            let Failure::Remote { error, native } = failure else {
                return Err("rejection not classified as native remote failure".into());
            };
            ensure(
                !error.is_empty() && !native.is_empty(),
                "lost native error evidence",
            )?;
            ensure(
                session.connection_token() == token,
                "clean remote rejection lost transport",
            )?;
            execute(&mut session, Ls::default()).await?;
            Ok(
                json!({"category":"Remote","native_bytes":native.len(),"clean_boundary_reused":true}),
            )
        }
    );
    case!(
        "finite-title-and-user-vars",
        json!({"commands":["set-window-title","set-user-vars","ls"]}),
        async {
            execute(
                &mut session,
                window::WindowCommand {
                    target: target.clone(),
                    operation: window::WindowOperation::Title {
                        title: Some("matrix finite title".into()),
                        temporary: false,
                    },
                },
            )
            .await?;
            execute(
                &mut session,
                window::UserVars {
                    target: target.clone(),
                    operations: vec![targets::Assignment::Set {
                        name: "matrix".into(),
                        value: "spaces and λ".into(),
                    }],
                },
            )
            .await?;
            let current = execute(&mut session, Ls::default()).await?;
            let window = current
                .iter()
                .flat_map(|o| &o.tabs)
                .flat_map(|t| &t.windows)
                .find(|w| w.id == Some(id))
                .ok_or("lost target")?;
            ensure(
                window.title.as_deref() == Some("matrix finite title"),
                "title not observed",
            )?;
            ensure(
                window.user_vars.get("matrix").map(String::as_str) == Some("spaces and λ"),
                "user variable not observed",
            )?;
            Ok(json!({"observed_title":window.title,"observed_var":window.user_vars["matrix"]}))
        }
    );
    case!(
        "nested-title-and-retirement",
        json!({"command":"action","action":"set_window_title matrix nested title"}),
        async {
            execute(
                &mut session,
                actions::Action {
                    target: target.clone(),
                    action: actions::KnownAction::SetWindowTitle("matrix nested title".into())
                        .into(),
                },
            )
            .await?;
            ensure(
                session.connection_token().is_none(),
                "nested dispatch transport retained",
            )?;
            let current = execute(&mut session, Ls::default()).await?;
            let title = current
                .iter()
                .flat_map(|o| &o.tabs)
                .flat_map(|t| &t.windows)
                .find(|w| w.id == Some(id))
                .and_then(|w| w.title.as_deref());
            ensure(
                title == Some("matrix nested title"),
                "nested title not observed",
            )?;
            Ok(json!({"dispatch_ack":true,"observed_title":title,"transport_retired":true}))
        }
    );
    case!(
        "colors-and-config-reload",
        json!({"commands":["set-colors","get-colors","load-config"],"reload_override":"foreground=#223344"}),
        async {
            let colors = BTreeMap::from([(
                "foreground".into(),
                appearance::ColorValue::Rgb(targets::Rgb::new(0x112233)?),
            )]);
            execute(
                &mut session,
                appearance::Appearance {
                    windows: targets::WindowSet {
                        match_window: Some(spec.clone()),
                        ..Default::default()
                    },
                    operation: appearance::AppearanceOperation::Colors {
                        colors,
                        configured: false,
                    },
                },
            )
            .await?;
            let configured = execute(
                &mut session,
                appearance::GetColors {
                    match_spec: Some(spec.clone()),
                    configured: false,
                },
            )
            .await?;
            ensure(
                has_color(&configured, "foreground", "#112233"),
                "set foreground not observed",
            )?;
            execute(
                &mut session,
                configuration::LoadConfig {
                    paths: vec![],
                    overrides: vec!["foreground=#223344".into()],
                    ignore_previous_overrides: false,
                },
            )
            .await?;
            let reloaded = execute(
                &mut session,
                appearance::GetColors {
                    match_spec: Some(spec.clone()),
                    configured: true,
                },
            )
            .await?;
            ensure(
                has_color(&reloaded, "foreground", "#223344"),
                "reload foreground not observed",
            )?;
            Ok(json!({"set_observed":"#112233","reloaded_config_observed":"#223344"}))
        }
    );
    case!(
        "custom-no-ui-kitten",
        json!({"command":"kitten","fixture":"no-ui.py","args":["one two","$MATRIX_VALUE","λ"]}),
        async {
            let values = vec![
                "one two".to_owned(),
                "$MATRIX_VALUE".to_owned(),
                "λ".to_owned(),
            ];
            let result = execute(
                &mut session,
                configuration::Kitten {
                    name: fixture.join("no-ui.py").to_string_lossy().into_owned(),
                    args: values.clone(),
                    target: target.clone(),
                },
            )
            .await?;
            let configuration::KittenResult::Text(text) = result else {
                return Err("kitten lost string return".into());
            };
            ensure(
                serde_json::from_str::<Vec<String>>(&text)? == values,
                "outer kitten changed literal argv",
            )?;
            ensure(
                session.connection_token().is_none(),
                "custom kitten transport retained",
            )?;
            let boolean = execute(
                &mut session,
                configuration::Kitten {
                    name: fixture.join("no-ui.py").to_string_lossy().into_owned(),
                    args: vec!["boolean".into()],
                    target: target.clone(),
                },
            )
            .await?;
            ensure(
                boolean == configuration::KittenResult::Boolean(true),
                "kitten lost bool return",
            )?;
            Ok(
                json!({"literal_argv":true,"string_return":true,"bool_return":true,"transport_retired":true}),
            )
        }
    );
    case!(
        "nested-kitten-token-boundaries",
        json!({"command":"action","action":"kitten","args":["record","private output path","one two","λ"]}),
        async {
            let output = fixture.join("nested-result.json");
            execute(
                &mut session,
                actions::Action {
                    target: target.clone(),
                    action: actions::KnownAction::Kitten {
                        name: fixture.join("no-ui.py").to_string_lossy().into_owned(),
                        args: vec![
                            "record".into(),
                            output.to_string_lossy().into_owned(),
                            "one two".into(),
                            "λ".into(),
                        ],
                    }
                    .into(),
                },
            )
            .await?;
            let result: Vec<String> =
                serde_json::from_str(&tokio::fs::read_to_string(output).await?)?;
            ensure(
                result == ["one two", "λ"],
                "nested argv token boundaries changed",
            )?;
            Ok(
                json!({"dispatch_ack":true,"fixture_effect_observed":true,"does_not_assert_no_server_expansion":true}),
            )
        }
    );
    case!(
        "submission-only-retirement",
        json!({"command":"focus-window","no_response":true}),
        async {
            let result = session
                .execute_with_options(
                    Submission(FocusWindow {
                        match_spec: spec.clone(),
                    }),
                    options(),
                )
                .await?;
            ensure(
                matches!(result, Completion::Submitted),
                "wrong suppression completion",
            )?;
            ensure(
                session.connection_token().is_none(),
                "suppressed-reply transport retained",
            )?;
            execute(&mut session, Ls::default()).await?;
            Ok(
                json!({"local_submission_only":true,"transport_retired":true,"next_explicit_command_succeeded":true}),
            )
        }
    );
    if version >= [0, 48, 0] {
        case!(
            "text-alternate-profile",
            json!({"command":"get-text","extent":"alternate","earliest_documented":[0,48,0]}),
            async {
                let text = execute(
                    &mut session,
                    GetText {
                        match_spec: Some(spec.clone()),
                        extent: TextExtent::Alternate,
                        ..Default::default()
                    },
                )
                .await?;
                Ok(json!({"text_bytes":text.len(),"no_alternate_content_asserted":true}))
            }
        );
    } else {
        record(
            "text-alternate-profile",
            json!({"command":"get-text","extent":"alternate"}),
            "skip",
            Duration::ZERO,
            json!({"gate":"documented since 0.48"}),
        );
    }
    let title_floor = catalog::capability("set-os-window-title")
        .and_then(|c| c.earliest_documented)
        .ok_or("missing OS-window title gate")?;
    if version >= title_floor {
        case!(
            "os-window-title-profile",
            json!({"command":"set-os-window-title","earliest_documented":title_floor}),
            async {
                execute(
                    &mut session,
                    window::WindowCommand {
                        target: target.clone(),
                        operation: window::WindowOperation::OsTitle {
                            title: Some("private matrix OS title".into()),
                        },
                    },
                )
                .await?;
                Ok(json!({"native_ack":true,"wm_title_not_observed":true}))
            }
        );
    } else {
        record(
            "os-window-title-profile",
            json!({"command":"set-os-window-title","earliest_documented":title_floor}),
            "skip",
            Duration::ZERO,
            json!({"gate":"earliest documented release"}),
        );
    }
    let screenshot_floor = catalog::capability("screenshot")
        .and_then(|c| c.earliest_documented)
        .ok_or("missing screenshot gate")?;
    if version >= screenshot_floor {
        case!(
            "screenshot-image-and-server-path",
            json!({"command":"screenshot","profiles":["PNG reply","private server file"]}),
            async {
                execute(
                    &mut session,
                    FocusWindow {
                        match_spec: spec.clone(),
                    },
                )
                .await?;
                tokio::time::sleep(Duration::from_millis(150)).await;
                let image = execute(
                    &mut session,
                    process::Screenshot {
                        target: process::ScreenshotTarget::Window(spec.clone()),
                    },
                )
                .await?;
                ensure(
                    image.len() > 8 && image.starts_with(b"\x89PNG\r\n\x1a\n"),
                    "bad image reply signature",
                )?;
                let path = fixture.join("screenshot.png");
                let saved = execute(
                    &mut session,
                    process::SaveScreenshot {
                        target: process::ScreenshotTarget::Window(spec.clone()),
                        server_path: targets::ServerPath::new(path.to_string_lossy())?,
                    },
                )
                .await?;
                ensure(
                    saved == path.to_string_lossy(),
                    "wrong save confirmation path",
                )?;
                let file = tokio::fs::read(path).await?;
                ensure(
                    file.len() > 8 && file.starts_with(b"\x89PNG\r\n\x1a\n"),
                    "bad saved PNG",
                )?;
                Ok(
                    json!({"reply_png_bytes":image.len(),"saved_png_bytes":file.len(),"private_file_verified":true}),
                )
            }
        );
    } else {
        record(
            "screenshot-image-and-server-path",
            json!({"command":"screenshot","earliest_documented":screenshot_floor}),
            "skip",
            Duration::ZERO,
            json!({"gate":"earliest documented release"}),
        );
    }
    ensure(failures == 0, &format!("{failures} profile cases failed"))?;
    Ok(())
}
