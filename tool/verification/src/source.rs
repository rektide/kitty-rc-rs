//! Pinned upstream source-contract verification.
//!
//! Two evidence profiles previously lived as Python AST scripts under
//! `design/`: exact constructor/parsers golden cases (capabilities) and the
//! upload constructor chunking contracts (uploads). Executing upstream's
//! actual methods is what makes upstream a *semantic* oracle — a textual grep
//! of the GPL source would be weaker evidence and a translation of the
//! algorithms into this repository is legally and methodologically wrong.
//! Therefore the minimal executor boundary stays Python
//! ([`fixtures/source_oracle.py`](../fixtures/source_oracle.py)); every case,
//! comparison, structural check and report below is Rust. Finite send-text
//! handler cases additionally exercise literal UTF-8/raw-byte semantics at the
//! stable and inspected HEAD revisions with synthetic selected-window sinks.

use base64::Engine as _;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const STABLE_REVISION: &str = "b2cb871254fbdac19a085188813c8dfb258b95cd";
const HEAD_REVISION: &str = "5bf3c2dc676efd5c4b4526eb135c922701565707";
const ORACLE: &str = include_str!("../fixtures/source_oracle.py");

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("preparing the oracle workspace failed")]
    Prepare {
        #[source]
        source: std::io::Error,
    },
    #[error("running the pinned-source oracle failed")]
    Oracle {
        #[source]
        source: std::io::Error,
    },
    #[error("oracle failed: {detail}")]
    OracleFailed { detail: String },
    #[error(
        "case {case} mismatched the pinned contract:\n  expected: {expected}\n  actual:   {actual}"
    )]
    Mismatch {
        case: String,
        expected: String,
        actual: String,
    },
    #[error("catalog inventory mismatch: native {native:?} vs modeled {modeled:?}")]
    InventoryMismatch {
        native: BTreeSet<String>,
        modeled: BTreeSet<String>,
    },
    #[error("case {case} failed inside the oracle: {detail}")]
    CaseFailed { case: String, detail: String },
}

pub struct SourceOptions {
    /// An upstream kovidgoyal/kitty checkout readable by `git`.
    pub checkout: PathBuf,
    /// Workspace root (for reading the modeled command catalog).
    pub root: PathBuf,
    pub python: PathBuf,
}

struct Oracle {
    results: Vec<Value>,
}

impl Oracle {
    /// Run every request in one oracle invocation and keep the aligned
    /// results for comparison.
    fn run(options: &SourceOptions, requests: &[Value]) -> Result<Self, SourceError> {
        let scratch = approved_temp("source-oracle");
        let oracle_path = scratch.join("source_oracle.py");
        std::fs::write(&oracle_path, ORACLE).map_err(|source| SourceError::Prepare { source })?;
        let document = json!({
            "checkout": options.checkout.display().to_string(),
            "revision": STABLE_REVISION,
            "requests": requests,
        });
        let mut child = std::process::Command::new(&options.python)
            .arg(&oracle_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|source| SourceError::Oracle { source })?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write as _;
            stdin
                .write_all(
                    serde_json::to_string(&document)
                        .unwrap_or_default()
                        .as_bytes(),
                )
                .map_err(|source| SourceError::Oracle { source })?;
        }
        let output = child
            .wait_with_output()
            .map_err(|source| SourceError::Oracle { source })?;
        if !output.status.success() {
            return Err(SourceError::OracleFailed {
                detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        let results: Vec<Value> =
            serde_json::from_slice(&output.stdout).map_err(|error| SourceError::OracleFailed {
                detail: error.to_string(),
            })?;
        Ok(Self { results })
    }

    fn value(&self, index: usize, case: &str) -> Result<&Value, SourceError> {
        let result = &self.results[index];
        if result["ok"] == Value::Bool(true) {
            Ok(&result["value"])
        } else {
            Err(SourceError::CaseFailed {
                case: case.into(),
                detail: result["error"].as_str().unwrap_or("<no error>").into(),
            })
        }
    }
}

/// Constructor golden cases: explicit opts/argv and independent expected
/// wire payloads (never this library's own serialization output).
fn constructor_cases() -> Vec<(&'static str, Value, Vec<&'static str>, Value)> {
    vec![
        (
            "action",
            json!({"self": false, "match": "id:7"}),
            vec!["goto_tab", "2"],
            json!({"action": "goto_tab 2", "self": false, "match_window": "id:7"}),
        ),
        (
            "close-window",
            json!({"self": false, "match": "id:8", "ignore_no_match": true}),
            vec![],
            json!({"self": false, "match": "id:8", "ignore_no_match": true}),
        ),
        (
            "detach-window",
            json!({"self": false, "match": "id:8", "target_tab": "new", "stay_in_tab": true}),
            vec![],
            json!({"self": false, "match": "id:8", "target_tab": "new", "stay_in_tab": true}),
        ),
        (
            "load-config",
            json!({"override": ["font_size=16", "background=#112233"], "ignore_overrides": true}),
            vec!["relative.conf", "/server/two.conf"],
            json!({"paths": ["relative.conf", "/server/two.conf"], "override": ["font_size=16", "background=#112233"], "ignore_overrides": true}),
        ),
        (
            "kitten",
            json!({"match": "id:11"}),
            vec!["custom.py", "--flag", "two words"],
            json!({"kitten": "custom.py", "args": ["--flag", "two words"], "match": "id:11"}),
        ),
        (
            "env",
            json!({}),
            vec!["A=b=c", "OLD"],
            json!({"env": {"A": "b=c", "OLD=": ""}}),
        ),
        (
            "resize-window",
            json!({"match": null, "self": true, "axis": "vertical", "increment": -3}),
            vec![],
            json!({"match": null, "self": true, "axis": "vertical", "increment": -3}),
        ),
        (
            "scroll-window",
            json!({"match": null}),
            vec!["0.5p-"],
            json!({"match": null, "self": true, "amount": [-0.5, "p"]}),
        ),
        (
            "select-window",
            json!({"match": "id:3", "self": false, "title": "pick one", "exclude_active": true, "reactivate_prev_tab": true}),
            vec![],
            json!({"match": "id:3", "self": false, "title": "pick one", "exclude_active": true, "reactivate_prev_tab": true}),
        ),
        (
            "send-key",
            json!({"match": null, "match_tab": null, "all": false, "exclude_active": true}),
            vec!["ctrl+a", "ctrl+b"],
            json!({"match": null, "match_tab": null, "all": false, "exclude_active": true, "keys": ["ctrl+a", "ctrl+b"]}),
        ),
        (
            "screenshot",
            json!({"match": "id:8", "match_tab": null}),
            vec![],
            json!({"match": "id:8", "match_tab": null, "output_path": ""}),
        ),
        (
            "set-user-vars",
            json!({"match": null}),
            vec!["project=a=b", "old"],
            json!({"match": null, "self": true, "var": ["project=a=b", "old"]}),
        ),
        (
            "set-font-size",
            json!({"all": false}),
            vec!["-2"],
            json!({"size": 2.0, "all": false, "increment_op": "-"}),
        ),
        (
            "set-window-title",
            json!({"match": "id:8", "temporary": false}),
            vec![],
            json!({"match": "id:8", "self": true, "temporary": false}),
        ),
        (
            "signal-child",
            json!({"match": null}),
            vec![],
            json!({"match": null, "self": true, "signals": ["SIGINT"]}),
        ),
    ]
}

/// Nested option parsers audited from `kitty/options/utils.py`.
fn parser_cases() -> Vec<(&'static str, &'static str, &'static str, Value)> {
    vec![
        ("goto_tab_parse", "goto_tab", "-1", json!(["goto_tab", [0]])),
        (
            "neighboring_window",
            "neighboring_window",
            "down",
            json!(["neighboring_window", ["bottom"]]),
        ),
        (
            "resize_window",
            "resize_window",
            "narrower 3",
            json!(["resize_window", ["narrower", 3]]),
        ),
        (
            "scroll_to_prompt",
            "scroll_to_prompt",
            "-1 2",
            json!(["scroll_to_prompt", [-1, 2]]),
        ),
        (
            "parse_change_font_size",
            "change_font_size",
            "current /2",
            json!(["change_font_size", [false, "/", 2.0]]),
        ),
        (
            "clear_terminal",
            "clear_terminal",
            "to_cursor_scroll all",
            json!(["clear_terminal", ["to_cursor_scroll", false]]),
        ),
        (
            "simple_parse",
            "set_window_title",
            "a 'quoted' title",
            json!(["set_window_title", ["a 'quoted' title"]]),
        ),
    ]
}

/// Independent wire/child-byte facts, not this library's own encoder/decoder
/// round trip. Window selection is a fixture; the real upstream handler owns
/// encoding dispatch, exclusion, paste wrapping/sanitizing and the write calls.
fn send_text_handler_cases() -> Vec<(&'static str, Value, Value)> {
    let wrapped = b"\x1b[200~\0\xff\x1b[201~";
    let long = vec![0xff; 1025];
    let cases = vec![
        (
            "literal-utf8-backslashes-colon",
            "text:\0ÿ\\n:base64".into(),
            false,
            "disable",
            vec![false],
            json!([["\0ÿ\\n:base64".as_bytes()]]),
        ),
        (
            "binary-nul-ff",
            "base64:AP8=".into(),
            false,
            "disable",
            vec![false],
            json!([[[0, 255]]]),
        ),
        (
            "utf8-via-base64",
            "base64:AMO/".into(),
            false,
            "disable",
            vec![false],
            json!([[[0, 195, 191]]]),
        ),
        (
            "binary-backslashes",
            "base64:XG4=".into(),
            false,
            "disable",
            vec![false],
            json!([[[92, 110]]]),
        ),
        (
            "empty-text",
            "text:".into(),
            false,
            "disable",
            vec![false],
            json!([[[]]]),
        ),
        (
            "empty-bytes",
            "base64:".into(),
            false,
            "disable",
            vec![false],
            json!([[[]]]),
        ),
        // 1024 is a reference-client generator choice, not this handler's
        // limit. This case makes no claim about dispatcher/transport ceilings.
        (
            "over-client-chunk-size",
            format!(
                "base64:{}",
                base64::engine::general_purpose::STANDARD.encode(&long)
            ),
            false,
            "disable",
            vec![false],
            json!([[long]]),
        ),
        (
            "exclude-active",
            "base64:AP8=".into(),
            true,
            "disable",
            vec![false, false],
            json!([[], [[0, 255]]]),
        ),
        (
            "auto-paste-off",
            "base64:AP8=".into(),
            false,
            "auto",
            vec![false],
            json!([[[0, 255]]]),
        ),
        (
            "auto-paste-on",
            "base64:AP8=".into(),
            false,
            "auto",
            vec![true],
            json!([[wrapped]]),
        ),
        (
            "forced-paste-sanitizes",
            "text:\u{1b}[201~X".into(),
            false,
            "enable",
            vec![false],
            json!([[b"\x1b[200~X\x1b[201~"]]),
        ),
    ];
    cases
        .into_iter()
        .map(|(name, data, exclude_active, bracketed_paste, bracketed_modes, writes)| {
            let request = json!({
                "kind": "send-text-handler",
                "payload": {"data": data, "exclude_active": exclude_active, "bracketed_paste": bracketed_paste},
                "bracketed_modes": bracketed_modes,
            });
            let expected = json!({"result": null, "writes": writes, "window_match_name": "match"});
            (name, request, expected)
        })
        .collect()
}

/// Scenario goldens record native state after each message/injected callback.
/// The helpers only format requests/observations; they implement no lifecycle.
fn broadcast_cases() -> Vec<(&'static str, Value, Value)> {
    let send = |payload: Value, selected: Value, origin: Option<u64>| json!({"kind": "send", "payload": payload, "selected": selected, "origin": origin});
    let state = |members: Value, cursors: Value, writes: Value, callbacks: Value| json!({"members": members, "cursors": cursors, "writes": writes, "callbacks": callbacks});
    let callbacks_a = json!({"99": {
        "removal": [["ClearSession", "a"]], "focus": [["FocusChangedSession", "a"]]
    }});
    let callbacks_ab = json!({"99": {
        "removal": [["ClearSession", "a"], ["ClearSession", "b"]],
        "focus": [["FocusChangedSession", "a"], ["FocusChangedSession", "b"]]
    }});
    let callbacks_ba = json!({"99": {
        "removal": [["ClearSession", "b"], ["ClearSession", "a"]],
        "focus": [["FocusChangedSession", "b"], ["FocusChangedSession", "a"]]
    }});
    let writes_ab = json!({"7": [[65]], "8": [[65]], "9": [[0, 255]]});
    let writes_abz = json!({"7": [[65]], "8": [[65], [90]], "9": [[0, 255]]});
    vec![
        (
            "broadcast-data-enrolls-and-end-clears-unselected-and-excluded-members",
            json!({"kind": "send-text-broadcast", "steps": [
                send(json!({"data": "text:A", "session_id": "caller", "exclude_active": false, "bracketed_paste": "disable"}), json!([7, 8]), None),
                send(json!({"data": "base64:AP8=", "session_id": "caller", "exclude_active": true, "bracketed_paste": "disable"}), json!([7, 9]), None),
                send(json!({"data": "text:Z", "exclude_active": false, "bracketed_paste": "disable"}), json!([8]), None),
                send(json!({"data": "session:end", "session_id": "caller", "exclude_active": true}), json!([7, 8]), None),
            ]}),
            json!([
                state(
                    json!({"caller": [7, 8]}),
                    json!([7, 8]),
                    json!({"7": [[65]], "8": [[65]]}),
                    json!({})
                ),
                state(
                    json!({"caller": [7, 8, 9]}),
                    json!([7, 8, 9]),
                    writes_ab,
                    json!({})
                ),
                state(
                    json!({"caller": [7, 8, 9]}),
                    json!([7, 8, 9]),
                    writes_abz.clone(),
                    json!({})
                ),
                state(json!({}), json!([]), writes_abz, json!({})),
            ]),
        ),
        (
            "broadcast-start-without-origin-installs-no-hooks-or-implicit-end",
            json!({"kind": "send-text-broadcast", "steps": [
                send(json!({"data": "session:start", "session_id": "caller", "exclude_active": false}), json!([8]), None),
                send(json!({"data": "session:start", "session_id": "caller", "exclude_active": false}), json!([9]), None),
            ]}),
            json!([
                state(json!({"caller": [8]}), json!([8]), json!({}), json!({})),
                state(
                    json!({"caller": [8, 9]}),
                    json!([8, 9]),
                    json!({}),
                    json!({})
                ),
            ]),
        ),
        (
            "broadcast-origin-hook-replacement-focus-and-removal",
            json!({"kind": "send-text-broadcast", "steps": [
                send(json!({"data": "session:start", "session_id": "a", "exclude_active": false}), json!([8]), Some(99)),
                send(json!({"data": "session:start", "session_id": "b", "exclude_active": false}), json!([9]), Some(99)),
                send(json!({"data": "session:start", "session_id": "a", "exclude_active": false}), json!([7]), Some(99)),
                {"kind": "focus", "origin": 99, "focused": false},
                {"kind": "focus", "origin": 99, "focused": true},
                send(json!({"data": "session:end", "session_id": "a", "exclude_active": true}), json!([7]), Some(99)),
                {"kind": "remove", "origin": 99},
            ]}),
            json!([
                state(json!({"a": [8]}), json!([8]), json!({}), callbacks_a),
                state(
                    json!({"a": [8], "b": [9]}),
                    json!([8, 9]),
                    json!({}),
                    callbacks_ab
                ),
                state(
                    json!({"a": [7, 8], "b": [9]}),
                    json!([7, 8, 9]),
                    json!({}),
                    callbacks_ba.clone()
                ),
                state(
                    json!({"a": [7, 8], "b": [9]}),
                    json!([]),
                    json!({}),
                    callbacks_ba.clone()
                ),
                state(
                    json!({"a": [7, 8], "b": [9]}),
                    json!([7, 8, 9]),
                    json!({}),
                    callbacks_ba.clone()
                ),
                state(
                    json!({"b": [9]}),
                    json!([9]),
                    json!({}),
                    callbacks_ba.clone()
                ),
                state(json!({}), json!([]), json!({}), callbacks_ba),
            ]),
        ),
    ]
}

/// Scan the modeled catalog for command names, mirroring the retired
/// verifier's extraction: `name: "…"` and `stable!("…")` occurrences.
pub fn modeled_catalog_names(catalog_text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let bytes = catalog_text.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        if catalog_text[position..].starts_with("name: \"") {
            let start = position + "name: \"".len();
            if let Some(end) = catalog_text[start..].find('"').map(|offset| start + offset) {
                names.insert(catalog_text[start..end].to_owned());
                position = end;
                continue;
            }
        }
        if catalog_text[position..].starts_with("stable!(") {
            let mut cursor = position + "stable!(".len();
            while matches!(
                catalog_text.as_bytes().get(cursor),
                Some(b' ' | b'\t' | b'\n' | b'\r')
            ) {
                cursor += 1;
            }
            if catalog_text[cursor..].starts_with('"') {
                let start = cursor + 1;
                if let Some(end) = catalog_text[start..].find('"').map(|offset| start + offset) {
                    names.insert(catalog_text[start..end].to_owned());
                    position = end;
                    continue;
                }
            }
        }
        position += 1;
    }
    names
}

fn constructor_request(name: &str, opts: &Value, args: &[&str]) -> Value {
    json!({
        "kind": "constructor",
        "path": format!("kitty/rc/{}.py", name.replace('-', "_")),
        "name": "message_to_kitty",
        "receiver": "fatal",
        "second": "namespace",
        "stream": false,
        "opts": opts,
        "args": args,
    })
}

/// Verify stable constructors/parsers/uploads plus finite send-text handlers
/// at stable and inspected HEAD; returns the evidence summary.
pub fn verify(options: &SourceOptions) -> Result<String, SourceError> {
    let constructors = constructor_cases();
    let parsers = parser_cases();
    let image_upload = image_upload_requests();
    let run_upload = run_upload_request();
    let send_text = send_text_handler_cases();
    let broadcasts = broadcast_cases();

    let mut requests = vec![json!({"kind": "inventory", "path": "kitty/rc"})];
    for (name, opts, args, _) in &constructors {
        requests.push(constructor_request(name, opts, args));
    }
    for (parser, action, rest, _) in &parsers {
        requests.push(json!({
            "kind": "parser",
            "path": "kitty/options/utils.py",
            "name": parser,
            "action": action,
            "rest": rest,
        }));
    }
    for request in &image_upload.0 {
        requests.push(request.clone());
    }
    requests.push(run_upload.0);
    for revision in [STABLE_REVISION, HEAD_REVISION] {
        for cases in [&send_text, &broadcasts] {
            for (_, request, _) in cases {
                let mut request = request.clone();
                request["revision"] = json!(revision);
                requests.push(request);
            }
        }
    }

    let oracle = Oracle::run(options, &requests)?;
    let mut index = 0;

    // Catalog inventory: every upstream RC command is modeled, except the
    // deliberately-raw HEAD-only resize-window-edge.
    let files: Vec<String> = serde_json::from_value(oracle.value(index, "inventory")?.clone())
        .map_err(|error| SourceError::OracleFailed {
            detail: error.to_string(),
        })?;
    index += 1;
    let native: BTreeSet<String> = files
        .iter()
        .filter(|file| file.ends_with(".py") && !matches!(file.as_str(), "base.py" | "__init__.py"))
        .map(|file| file.trim_end_matches(".py").replace('_', "-"))
        .collect();
    let catalog_text =
        std::fs::read_to_string(options.root.join("protocol/commands/src/catalog.rs"))
            .map_err(|source| SourceError::Prepare { source })?;
    let mut modeled = modeled_catalog_names(&catalog_text);
    modeled.remove("resize-window-edge");
    if native != modeled {
        return Err(SourceError::InventoryMismatch { native, modeled });
    }
    let native_count = native.len();

    for (name, _, _, expected) in &constructors {
        let actual = oracle.value(index, name)?.clone();
        index += 1;
        // JSON-normalized equality, exactly like the retired verifier.
        if actual != *expected {
            return Err(SourceError::Mismatch {
                case: (*name).into(),
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
    }
    for (parser, _, _, expected) in &parsers {
        let actual = oracle.value(index, parser)?.clone();
        index += 1;
        if actual != *expected {
            return Err(SourceError::Mismatch {
                case: (*parser).into(),
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
    }

    // Upload constructors: structural chunking contracts.
    let body = image_upload.1.clone();
    for name in ["set_background_image", "set_window_logo"] {
        let frames = oracle.value(index, name)?.clone();
        index += 1;
        verify_image_frames(name, &frames, &body, &image_upload.2[name])?;
    }
    let run_frames = oracle.value(index, "run message_to_kitty")?.clone();
    index += 1;
    verify_run_frames(&run_frames, &run_upload.1)?;
    for revision in [STABLE_REVISION, HEAD_REVISION] {
        for cases in [&send_text, &broadcasts] {
            for (name, _, expected) in cases {
                let case = format!("send-text {name} at {revision}");
                let actual = oracle.value(index, &case)?;
                index += 1;
                if actual != expected {
                    return Err(SourceError::Mismatch {
                        case,
                        expected: expected.to_string(),
                        actual: actual.to_string(),
                    });
                }
            }
        }
    }
    // Every request must have been consumed: guards request/result drift.
    assert_eq!(index, requests.len(), "oracle request/result misalignment");

    Ok(format!(
        "Pinned source verified: {native_count} commands, {} constructor cases, {} nested parsers, upload image/run chunk contracts; {} finite send-text handler cases and {} broadcast lifecycle scenarios at each of stable {STABLE_REVISION} and inspected HEAD {HEAD_REVISION}",
        constructors.len(),
        parsers.len(),
        send_text.len(),
        broadcasts.len(),
    ))
}

struct ImageUploadRequests(
    pub Vec<Value>,
    pub Vec<u8>,
    pub BTreeMap<&'static str, Value>,
);

fn approved_temp(subdirectory: &str) -> PathBuf {
    let directory = Path::new("/tmp/opencode").join(subdirectory);
    std::fs::create_dir_all(&directory).ok();
    directory
}

fn image_upload_requests() -> ImageUploadRequests {
    let body: Vec<u8> = b"\x89PNG\r\n\x1a\n"
        .iter()
        .copied()
        .chain((0..256u16).map(|n| n as u8).cycle().take(256 * 17))
        .collect();
    let png_path = approved_temp("source-fixture").join("fixture.png");
    std::fs::write(&png_path, &body).ok();
    let path_text = png_path.display().to_string();
    let mut requests = Vec::new();
    let mut metadata = BTreeMap::new();
    for (file, opts, expected) in [
        (
            "set_background_image",
            json!({"match": "title:🦀", "all": true, "configured": true, "layout": "cscaled"}),
            json!({"match": "title:🦀", "all": true, "configured": true, "layout": "cscaled"}),
        ),
        (
            "set_window_logo",
            json!({"match": null, "self": true, "alpha": 0.3, "position": "bottom-right"}),
            json!({"match": null, "self": true, "alpha": 0.3, "position": "bottom-right"}),
        ),
    ] {
        requests.push(json!({
            "kind": "constructor",
            "path": format!("kitty/rc/{file}.py"),
            "name": "message_to_kitty",
            "receiver": "empty",
            "second": "none",
            "stream": true,
            "opts": opts,
            "args": [path_text],
        }));
        metadata.insert(file, expected);
    }
    ImageUploadRequests(requests, body, metadata)
}

struct RunUploadRequest(pub Value, pub Vec<u8>);

fn run_upload_request() -> RunUploadRequest {
    let body: Vec<u8> = (0..256u16)
        .map(|n| n as u8)
        .cycle()
        .take(256 * 33)
        .collect();
    let request = json!({
        "kind": "constructor",
        "path": "kitty/rc/run.py",
        "name": "message_to_kitty",
        "receiver": "empty",
        "second": "none",
        "stream": true,
        "opts": {"env": ["X=a=b"], "allow_remote_control": true, "remote_control_password": ["pw permit-*"]},
        "args": ["echo", "a🦀\0binary"],
        "stdin": base64::engine::general_purpose::STANDARD.encode(&body),
    });
    RunUploadRequest(request, body)
}

fn decode_frame_data(frame: &Value) -> Result<Vec<u8>, SourceError> {
    let text = frame["data"].as_str().unwrap_or_default();
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|error| SourceError::OracleFailed {
            detail: error.to_string(),
        })
}

fn verify_image_frames(
    name: &str,
    frames: &Value,
    body: &[u8],
    expected_metadata: &Value,
) -> Result<(), SourceError> {
    let frames = frames.as_array().ok_or_else(|| SourceError::CaseFailed {
        case: name.into(),
        detail: "frames not a list".into(),
    })?;
    let decoded: Vec<Vec<u8>> = frames
        .iter()
        .map(decode_frame_data)
        .collect::<Result<_, _>>()?;
    let empty = decoded.iter().filter(|data| data.is_empty()).count();
    let mismatch = |detail: String| SourceError::CaseFailed {
        case: name.into(),
        detail,
    };
    if empty != 1 || !decoded.last().is_some_and(|data| data.is_empty()) {
        return Err(mismatch(format!(
            "expected exactly one final empty data frame, saw {empty}"
        )));
    }
    let joined: Vec<u8> = decoded.iter().flatten().copied().collect();
    if joined != body {
        return Err(mismatch(
            "frame payloads do not reconstruct the fixture image".into(),
        ));
    }
    if decoded.iter().any(|data| data.len() > 512) {
        return Err(mismatch("image frame exceeds 512-byte chunks".into()));
    }
    let identity = frames[0].get("stream_id").cloned();
    for frame in frames {
        if frame.get("stream_id") != identity.as_ref() {
            return Err(mismatch("stream identity changed mid-upload".into()));
        }
        let mut remainder = frame.clone();
        if let Some(object) = remainder.as_object_mut() {
            object.remove("stream_id");
            object.remove("data");
        }
        if &remainder != expected_metadata {
            return Err(SourceError::Mismatch {
                case: name.into(),
                expected: expected_metadata.to_string(),
                actual: remainder.to_string(),
            });
        }
    }
    Ok(())
}

fn verify_run_frames(frames: &Value, body: &[u8]) -> Result<(), SourceError> {
    let frames = frames.as_array().ok_or_else(|| SourceError::CaseFailed {
        case: "run message_to_kitty".into(),
        detail: "frames not a list".into(),
    })?;
    let decoded: Vec<Vec<u8>> = frames
        .iter()
        .map(decode_frame_data)
        .collect::<Result<_, _>>()?;
    let lengths: Vec<usize> = decoded.iter().map(Vec::len).collect();
    if lengths != vec![4096, 4096, 256] {
        return Err(SourceError::CaseFailed {
            case: "run message_to_kitty".into(),
            detail: format!("unexpected chunk lengths {lengths:?}, expected [4096, 4096, 256]"),
        });
    }
    let joined: Vec<u8> = decoded.iter().flatten().copied().collect();
    if joined != body {
        return Err(SourceError::CaseFailed {
            case: "run message_to_kitty".into(),
            detail: "chunks do not reconstruct stdin body".into(),
        });
    }
    for frame in frames {
        if frame["cmdline"].as_array().map(|values| {
            values
                .iter()
                .map(|value| value.as_str().unwrap_or_default())
                .collect::<Vec<_>>()
        }) != Some(vec!["echo", "a🦀\0binary"])
        {
            return Err(SourceError::CaseFailed {
                case: "run message_to_kitty".into(),
                detail: "cmdline changed".into(),
            });
        }
        for key in ["env", "allow_remote_control", "remote_control_password"] {
            if frame.get(key).is_none() {
                return Err(SourceError::CaseFailed {
                    case: "run message_to_kitty".into(),
                    detail: format!("missing passthrough option {key}"),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_scan_extracts_name_and_stable_entries() {
        let catalog = r#"
        stable!("ls", ...)
        stable!("close-window")
        Capability { name: "screenshot", .. }
        stable!( "detach-window" )
        irrelevant_name: "nope",
        "#;
        let names = modeled_catalog_names(catalog);
        // `irrelevant_name:` still contains the `name: "` pattern: the scan
        // deliberately mirrors the retired verifier's semantics exactly.
        assert_eq!(
            names,
            BTreeSet::from(
                ["ls", "close-window", "screenshot", "detach-window", "nope"].map(String::from)
            )
        );
    }

    #[test]
    fn golden_tables_cover_the_audited_contract_surface() {
        assert_eq!(constructor_cases().len(), 15);
        assert_eq!(parser_cases().len(), 7);
    }
}
