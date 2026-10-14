//! Test fixture binary standing in for a prebuilt example binary. Verifies
//! the runner's environment contract, emits a JSON case record and exits
//! with `$FAKE_EXAMPLE_EXIT` (default 0).
//!
//! Contract checked (mirrors the real examples' private-fixture rules):
//! `KITTY_RC_MATRIX_VERSION`, `…_PROTECTION` and `…_MARKER` are present;
//! no ambient `KITTY_*` endpoint/key variables leaked in; the socket
//! argument lives inside a recognized private fixture directory. The
//! complete sanitized environment is dumped to `$FAKE_EXAMPLE_DUMP` for
//! caller-side assertions.
use std::collections::BTreeMap;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 3, "expected SOCKET KEY PASSWORD arguments");
    let socket = std::path::Path::new(&args[0]);
    let parent = socket.parent().expect("socket parent");
    let parent_name = parent
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    assert!(
        socket.file_name().is_some_and(|name| name == "rc.sock")
            && (parent_name.starts_with("kitty-rc-fixture-")
                || parent_name.starts_with("kitty-rc-live-")
                || parent_name.starts_with("kitty-private-upload-")),
        "refusing a non-conformance endpoint {parent_name}",
    );
    let env: BTreeMap<String, String> = std::env::vars().collect();
    for required in [
        "KITTY_RC_MATRIX_VERSION",
        "KITTY_RC_MATRIX_PROTECTION",
        "KITTY_RC_MATRIX_MARKER",
    ] {
        assert!(env.contains_key(required), "missing {required}");
    }
    for forbidden in [
        "KITTY_PUBLIC_KEY",
        "KITTY_PID",
        "KITTY_WINDOW_ID",
        "KITTY_LISTEN_ON",
        "DISPLAY",
    ] {
        assert!(
            !env.contains_key(forbidden),
            "ambient {forbidden} leaked into the example environment"
        );
    }
    // WAYLAND_DISPLAY is the private fixture's own socket, never ambient.
    assert!(
        env.get("WAYLAND_DISPLAY")
            .is_some_and(|display| display.starts_with("wl-")),
        "WAYLAND_DISPLAY is not the private fixture's socket: {:?}",
        env.get("WAYLAND_DISPLAY")
    );
    assert_eq!(
        env.get("KITTY_RC_MATRIX_PROTECTION").map(String::as_str),
        Some("encrypted"),
        "example contract is encrypted-only here"
    );
    if let Some(dump) = env.get("FAKE_EXAMPLE_DUMP") {
        let mut contents = String::new();
        for (key, value) in &env {
            contents.push_str(&format!("{key}={value}\n"));
        }
        std::fs::write(dump, contents).expect("dump environment");
    }
    println!(
        "{}",
        serde_json::json!({"case": "fake-example-case", "status": "pass", "detail": {"args": args.len()}})
    );
    let _ = std::io::stdout().flush();
    let code: i32 = env
        .get("FAKE_EXAMPLE_EXIT")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    std::process::exit(code);
}
