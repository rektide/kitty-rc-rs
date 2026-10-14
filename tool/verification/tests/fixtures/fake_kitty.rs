//! Test fixture binary standing in for a kitty server. Never a real
//! terminal: it serves exactly one already-composed private fixture.
//!
//! `--version` reports a fixed release. `--config P --listen-on unix:S
//! CHILD…` validates the config is a recognized private fixture config,
//! binds the socket, then launches the real fixture child with
//! `KITTY_PUBLIC_KEY` exported. `FAKE_KITTY_MODE` selects behavior:
//! `ok` (default), `fail` (exit immediately), `hang` (never serve).
use std::os::unix::net::UnixListener;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = std::env::var("FAKE_KITTY_MODE").unwrap_or_else(|_| "ok".into());
    if args == ["--version"] {
        println!("kitty 0.49.2 created by Test Fixture");
        return;
    }
    if mode == "fail" {
        eprintln!("fake kitty: configured startup failure");
        std::process::exit(1);
    }
    let config_index = args
        .iter()
        .position(|argument| argument == "--config")
        .expect("--config");
    let config = std::path::Path::new(&args[config_index + 1]);
    let listen_index = args
        .iter()
        .position(|argument| argument == "--listen-on")
        .expect("--listen-on");
    let socket = args[listen_index + 1]
        .strip_prefix("unix:")
        .expect("unix socket")
        .to_owned();
    let parent = config.parent().expect("config parent");
    let parent_name = parent
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    assert!(
        config.file_name().is_some_and(|name| name == "kitty.conf")
            && (parent_name.starts_with("kitty-rc-fixture-")
                || parent_name.starts_with("kitty-rc-live-")
                || parent_name.starts_with("kitty-private-upload-")),
        "refusing a non-conformance config {parent_name}",
    );
    if mode == "hang" {
        std::thread::sleep(std::time::Duration::from_secs(120));
        return;
    }
    let listener = UnixListener::bind(&socket).expect("bind fixture socket");
    let child_arguments: Vec<String> = args[listen_index + 2..].to_vec();
    assert!(
        !child_arguments.is_empty(),
        "expected the fixture child command"
    );
    let mut command = std::process::Command::new(&child_arguments[0]);
    command
        .args(&child_arguments[1..])
        .env("KITTY_PUBLIC_KEY", "1:test-fixture-public-key");
    let mut child = command.spawn().expect("spawn fixture child");
    let _listener = listener; // hold the socket open for the fixture lifetime
    let status = child.wait().expect("wait fixture child");
    std::process::exit(status.code().unwrap_or(1));
}
