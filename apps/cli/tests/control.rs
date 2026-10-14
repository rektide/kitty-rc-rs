//! Actual executable journeys; private sockets, selected subprocess env only.
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = PathBuf::from(format!(
            "/tmp/opencode/kitty-control-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_kitty-rc"));
        command
            .env_clear()
            .env("HOME", &self.0)
            .env("XDG_STATE_HOME", &self.0)
            .env("XDG_CONFIG_HOME", &self.0);
        command
    }
    fn server(&self, replies: Vec<Value>) -> Server {
        Server::start(self.0.join("rc.sock"), replies, None)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
struct Server {
    path: PathBuf,
    requests: Arc<Mutex<Vec<Value>>>,
    worker: Option<thread::JoinHandle<()>>,
}
fn request(stream: &mut UnixStream) -> Option<Value> {
    let mut bytes = Vec::new();
    let mut byte = [0];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => return None,
            Ok(_) => bytes.push(byte[0]),
            Err(error) => panic!("fake-peer read: {error}"),
        }
        if bytes.ends_with(b"\x1b\\") {
            break;
        }
        assert!(bytes.len() < 1_048_576);
    }
    assert!(bytes.starts_with(b"\x1bP@kitty-cmd"));
    Some(serde_json::from_slice(&bytes[12..bytes.len() - 2]).unwrap())
}
impl Server {
    fn start(
        path: PathBuf,
        replies: Vec<Value>,
        secret: Option<x25519_dalek::StaticSecret>,
    ) -> Self {
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let saved = requests.clone();
        let worker = thread::spawn(move || {
            let mut stream: Option<UnixStream> = None;
            for reply in replies {
                let received = loop {
                    if stream.is_none() {
                        let until = Instant::now() + Duration::from_secs(5);
                        loop {
                            match listener.accept() {
                                Ok((new, _)) => {
                                    new.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                                    stream = Some(new);
                                    break;
                                }
                                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                    assert!(Instant::now() < until, "CLI never connected");
                                    thread::sleep(Duration::from_millis(2));
                                }
                                Err(error) => panic!("accept: {error}"),
                            }
                        }
                    }
                    if let Some(request) = request(stream.as_mut().unwrap()) {
                        break request;
                    }
                    stream = None;
                };
                let received = if let Some(secret) = &secret {
                    let envelope = serde_json::from_value(received).unwrap();
                    kitty_rc_crypto::decrypt_envelope(&envelope, secret).unwrap()
                } else {
                    received
                };
                saved.lock().unwrap().push(received);
                let wire = format!("\x1bP@kitty-cmd{}\x1b\\", reply);
                // Fragment framing deliberately, rather than mirroring one local write.
                stream
                    .as_mut()
                    .unwrap()
                    .write_all(&wire.as_bytes()[..5])
                    .unwrap();
                stream
                    .as_mut()
                    .unwrap()
                    .write_all(&wire.as_bytes()[5..])
                    .unwrap();
            }
        });
        Self {
            path,
            requests,
            worker: Some(worker),
        }
    }
    fn finish(mut self) -> Vec<Value> {
        self.worker.take().unwrap().join().unwrap();
        fs::remove_file(&self.path).unwrap();
        self.requests.lock().unwrap().clone()
    }
}
fn listing() -> Value {
    json!([{"id":10,"is_focused":true,"tabs":[{"id":20,"is_active":true,"windows":[
        {"id":30,"title":"shell","pid":123,"cwd":"/fixture","cmdline":["zsh"],"is_active":true,"is_focused":false,"foreground_processes":[{"pid":234,"cwd":"/work","cmdline":["vim","file"]}]},
        {"id":31,"title":"focused","is_active":false,"is_focused":true}
    ]}]}])
}
fn reply(listing: Value) -> Value {
    json!({"ok":true,"data":listing.to_string()})
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn failure(output: &Output, needle: &str) {
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(needle),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "failure must not fabricate successful stdout"
    );
}

#[test]
fn list_aliases_share_engine_and_useful_legacy_output_fields() {
    for alias in ["list-windows", "ls", "list"] {
        let scratch = Scratch::new();
        let server = scratch.server(vec![reply(listing())]);
        let out = scratch
            .command()
            .args([alias, "--to"])
            .arg(&server.path)
            .arg("--json")
            .output()
            .unwrap();
        success(&out);
        let rows: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(rows[0]["id"], 30);
        assert_eq!(rows[0]["foreground_processes"][0]["cmdline"][0], "vim");
        assert_eq!(server.finish()[0]["cmd"], "ls");
    }
    let scratch = Scratch::new();
    let server = scratch.server(vec![reply(listing())]);
    let out = scratch
        .command()
        .args(["ls", "--to"])
        .arg(&server.path)
        .output()
        .unwrap();
    success(&out);
    let text = String::from_utf8(out.stdout).unwrap();
    for field in ["Window 30", "shell", "/fixture", "zsh", "vim", "/work"] {
        assert!(text.contains(field));
    }
    server.finish();
}
#[test]
fn active_focus_and_explicit_tab_os_targets_use_different_facts() {
    for (args, expected) in [
        (vec!["active"], 31),
        (vec!["a", "--tab", "20"], 30),
        (vec!["active-window", "--os-window", "10"], 30),
    ] {
        let scratch = Scratch::new();
        let server = scratch.server(vec![reply(listing())]);
        let out = scratch
            .command()
            .args(args)
            .arg("--to")
            .arg(&server.path)
            .arg("--json")
            .output()
            .unwrap();
        success(&out);
        assert_eq!(
            serde_json::from_slice::<Value>(&out.stdout).unwrap()["id"],
            expected
        );
        server.finish();
    }
}
#[test]
fn absent_ambiguous_missing_and_filtered_focus_are_not_global_winners() {
    let mut missing = listing();
    missing[0]["tabs"][0]["windows"][0]
        .as_object_mut()
        .unwrap()
        .remove("is_focused");
    let mut ambiguous = listing();
    ambiguous[0]["tabs"][0]["windows"][0]["is_focused"] = json!(true);
    for (data, args, needle) in [
        (json!([]), vec!["a"], "no active/focused"),
        (missing, vec!["a"], "fact missing"),
        (ambiguous, vec!["a"], "ambiguous"),
        (listing(), vec!["a", "--tab", "99"], "missing or ambiguous"),
        (listing(), vec!["a", "--match", "id:31"], "unfiltered"),
    ] {
        let scratch = Scratch::new();
        let server = scratch.server(vec![reply(data)]);
        let out = scratch
            .command()
            .args(args)
            .arg("--to")
            .arg(&server.path)
            .arg("--json")
            .output()
            .unwrap();
        failure(&out, needle);
        server.finish();
    }
}
#[test]
fn remote_refusal_and_malformed_typed_data_are_actionable_nonzero() {
    for (data, needle) in [
        (
            json!({"ok":false,"error":"permission denied"}),
            "remote error",
        ),
        (json!({"ok":true}), "typed decode"),
        (reply(json!([{"id":10}])), "typed decode"),
        (reply(json!("wrong")), "typed decode"),
    ] {
        let scratch = Scratch::new();
        let server = scratch.server(vec![data]);
        let out = scratch
            .command()
            .args(["ls", "--to"])
            .arg(&server.path)
            .arg("--json")
            .output()
            .unwrap();
        failure(&out, needle);
        server.finish();
    }
}
#[test]
fn goto_sends_stable_id_and_never_retries_refusal() {
    for allowed in [true, false] {
        let scratch = Scratch::new();
        let server = scratch.server(vec![if allowed {
            json!({"ok":true})
        } else {
            json!({"ok":false,"error":"cannot focus"})
        }]);
        let out = scratch
            .command()
            .args(["g", "31", "--to"])
            .arg(&server.path)
            .arg("--json")
            .output()
            .unwrap();
        if allowed {
            success(&out);
            assert_eq!(
                serde_json::from_slice::<Value>(&out.stdout).unwrap()["focused_window_id"],
                31
            );
        } else {
            failure(&out, "cannot focus");
        }
        let sent = server.finish();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0]["cmd"], "focus-window");
        assert_eq!(sent[0]["payload"]["match"], "id:31");
    }
}
#[test]
fn finite_watch_reuses_and_releases_records_and_reports_sampled_absence() {
    let scratch = Scratch::new();
    let mut later = listing();
    later[0]["tabs"][0]["windows"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    let mut replies = vec![reply(listing())];
    replies.extend((0..70).map(|_| reply(later.clone())));
    let server = scratch.server(replies);
    let start = Instant::now();
    let out = scratch
        .command()
        .args(["w", "--count", "71", "--interval-ms", "1", "--to"])
        .arg(&server.path)
        .arg("--json")
        .output()
        .unwrap();
    success(&out);
    assert!(start.elapsed() >= Duration::from_millis(60));
    let samples: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(samples.len(), 71);
    assert_eq!(
        samples[1]["changes"][0],
        json!({"kind":"NoLongerReturned","id":30})
    );
    assert_eq!(samples[70]["sequence"], 71);
    assert!(!String::from_utf8_lossy(&out.stderr).contains("shutdown operation"));
    assert_eq!(server.finish().len(), 71);
}
#[test]
fn endpoint_config_env_explicit_precedence_and_help_do_not_touch_human_paths() {
    let scratch = Scratch::new();
    let server = scratch.server(vec![reply(listing())]);
    let config = scratch.0.join("config.json");
    fs::write(&config, r#"{"endpoints":["fd:99"],"auth":"plain"}"#).unwrap();
    let out = scratch
        .command()
        .env("KITTY_LISTEN_ON", "fd:9")
        .env("KITTY_PID", "bad")
        .arg("--config")
        .arg(&config)
        .arg("--to")
        .arg(&server.path)
        .args(["ls", "--json", "--verbose"])
        .output()
        .unwrap();
    success(&out);
    assert!(String::from_utf8_lossy(&out.stderr).contains("Explicit"));
    server.finish();
    let server = scratch.server(vec![reply(listing())]);
    let out = scratch
        .command()
        .env("KITTY_LISTEN_ON", format!("unix:{}", server.path.display()))
        .arg("--config")
        .arg(&config)
        .args(["ls", "--json"])
        .output()
        .unwrap();
    success(&out);
    server.finish();
    fs::write(&config, r#"{"endpoints":["unix:/one","unix:/two"]}"#).unwrap();
    failure(
        &scratch
            .command()
            .arg("--config")
            .arg(&config)
            .arg("ls")
            .output()
            .unwrap(),
        "ambiguous",
    );
    failure(&scratch.command().arg("ls").output().unwrap(), "--to");
    for address in ["fd:9", "unix:@abstract"] {
        failure(
            &scratch
                .command()
                .env("KITTY_LISTEN_ON", address)
                .arg("ls")
                .output()
                .unwrap(),
            "unsupported",
        );
    }
    let help = scratch.command().arg("--help").output().unwrap();
    success(&help);
    assert!(String::from_utf8_lossy(&help.stdout).contains("list-windows"));
    assert_eq!(
        scratch
            .command()
            .args(["ls", "--timeout-ms", "0"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn encrypted_shared_store_and_password_flag_precedence_protect_exactly_once() {
    let scratch = Scratch::new();
    let secret = x25519_dalek::StaticSecret::from([7u8; 32]);
    let key = format!(
        "1:{}",
        base85::encode(x25519_dalek::PublicKey::from(&secret).as_bytes())
    );
    let server = Server::start(
        scratch.0.join("rc.sock"),
        vec![reply(listing())],
        Some(secret),
    );
    let store = scratch.0.join("pubkey.tsv");
    let original = format!("7\t4\t{key}\n");
    fs::write(&store, &original).unwrap();
    let config = scratch.0.join("config.json");
    fs::write(
        &config,
        r#"{"auth":"plain","key_file":"/never-read","password_file":"/never-read"}"#,
    )
    .unwrap();
    let out = scratch
        .command()
        .env("KITTY_RC_AUTH", "invalid")
        .env("KITTY_RC_KEY_FILE", "/never-read")
        .env("KITTY_RC_PASSWORD_FILE", "/never-read")
        .env("SELECTED_PASSWORD", "fixture-secret")
        .env("KITTY_RC_PASSWORD", "wrong-secret")
        .arg("--config")
        .arg(&config)
        .arg("--to")
        .arg(&server.path)
        .args(["--auth", "encrypted", "--pid", "7", "--key-store"])
        .arg(&store)
        .args([
            "--password-env",
            "SELECTED_PASSWORD",
            "--verbose",
            "ls",
            "--json",
        ])
        .output()
        .unwrap();
    success(&out);
    assert!(!String::from_utf8_lossy(&out.stderr).contains("fixture-secret"));
    assert_eq!(fs::read_to_string(&store).unwrap(), original);
    let request = server.finish().remove(0);
    assert_eq!(request["password"], "fixture-secret");
    assert_eq!(request["cmd"], "ls");
    assert!(request.get("encrypted").is_none());
}
#[test]
fn malformed_auth_sources_do_not_downgrade_or_leak_passwords() {
    let scratch = Scratch::new();
    let key = scratch.0.join("bad-key");
    fs::write(&key, "bad-key\n").unwrap();
    let out = scratch
        .command()
        .env("KITTY_RC_PASSWORD", "fixture-secret")
        .args(["--to", "/nonexistent", "--auth", "encrypted", "--key-file"])
        .arg(&key)
        .arg("ls")
        .output()
        .unwrap();
    failure(&out, "invalid public key");
    assert!(!String::from_utf8_lossy(&out.stderr).contains("fixture-secret"));
    let store = scratch.0.join("store");
    fs::write(&store, "malformed without newline").unwrap();
    let out = scratch
        .command()
        .args([
            "--to",
            "/nonexistent",
            "--auth",
            "encrypted",
            "--pid",
            "7",
            "--key-store",
        ])
        .arg(&store)
        .arg("ls")
        .output()
        .unwrap();
    failure(&out, "malformed key store");
    assert_eq!(
        fs::read_to_string(&store).unwrap(),
        "malformed without newline"
    );
    failure(
        &scratch
            .command()
            .env("KITTY_PUBLIC_KEY", "not-a-key")
            .env("KITTY_LISTEN_ON", "unix:/other")
            .args(["--to", "/selected", "--auth", "encrypted", "ls"])
            .output()
            .unwrap(),
        "not AccessBinding",
    );
}
#[test]
fn ctrl_c_during_accepted_watch_read_reports_uncertainty_and_closes_driver() {
    let scratch = Scratch::new();
    let path = scratch.0.join("cancel.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let (sent, received) = std::sync::mpsc::channel();
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let request = request(&mut stream).unwrap();
        sent.send(request).unwrap();
        let mut byte = [0];
        assert_eq!(stream.read(&mut byte).unwrap(), 0);
    });
    let mut child = scratch
        .command()
        .args(["w", "--to"])
        .arg(&path)
        .args(["--json", "--timeout-ms", "2000"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert_eq!(
        received.recv_timeout(Duration::from_secs(5)).unwrap()["cmd"],
        "ls"
    );
    assert!(
        Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let status = child.wait().unwrap();
    assert!(!status.success());
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("unresolved=true"), "{stderr}");
    assert!(stderr.contains("Ctrl-C"));
    assert!(out.stdout.is_empty());
    worker.join().unwrap();
}
