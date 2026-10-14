//! Lifecycle tests for the private fixture engine and matrix composition,
//! using the Rust fixture binaries (fake-kitty, fake-weston, fake-example,
//! fake-cli) and the real `kitty-rc-verify-child`. No Python is involved at
//! any layer: these exercise genuine process supervision, environment
//! sanitation, deadline bounds and exit-code propagation.

use kitty_rc_verify::Protection;
use kitty_rc_verify::env::sanitized_ambient;
use kitty_rc_verify::fixture::{self, FixtureError, FixtureOptions};
use kitty_rc_verify::matrix::{self, Binaries, MatrixOptions};
use kitty_rc_verify::smoke::{self, SmokeOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn bin(name: &str) -> PathBuf {
    match name {
        "kitty" => env!("CARGO_BIN_EXE_fake-kitty").into(),
        "weston" => env!("CARGO_BIN_EXE_fake-weston").into(),
        "example" => env!("CARGO_BIN_EXE_fake-example").into(),
        "cli" => env!("CARGO_BIN_EXE_fake-cli").into(),
        "child" => env!("CARGO_BIN_EXE_kitty-rc-verify-child").into(),
        other => panic!("unknown fixture binary {other}"),
    }
}

/// A base environment with poisoned ambient entrances: every spawned child
/// must have them stripped.
fn poisoned_base() -> BTreeMap<String, String> {
    let mut env = sanitized_ambient();
    env.insert("KITTY_PUBLIC_KEY".into(), "human-key".into());
    env.insert("KITTY_LISTEN_ON".into(), "human.sock".into());
    env.insert("DISPLAY".into(), ":0".into());
    env
}

fn options(plain: bool) -> FixtureOptions {
    FixtureOptions {
        kitty: bin("kitty"),
        weston: bin("weston"),
        child: bin("child"),
        password: "fixture-test-secret".into(),
        plain,
        augment_config: true,
        marker: "kitty-rc private fixture".into(),
        startup_timeout: Duration::from_secs(30),
        child_sleep: Duration::from_secs(120),
        base_env: Some(poisoned_base()),
    }
}

#[tokio::test]
async fn fixture_publishes_key_serves_socket_and_tears_down() {
    let parent = tempfile::tempdir().unwrap();
    let fixture = fixture::start(options(false), parent.path()).await.unwrap();
    assert!(
        fixture
            .dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kitty-rc-fixture-")
    );
    assert!(fixture.socket.exists());
    assert_eq!(fixture.key, "1:test-fixture-public-key");
    let config = std::fs::read_to_string(fixture.dir.join("kitty.conf")).unwrap();
    assert!(config.contains("allow_remote_control password"));
    assert!(config.contains("remote_control_password fixture-test-secret"));
    assert!(fixture.dir.join("no-ui.py").exists());

    let mut env = fixture.environment();
    env.insert("KITTY_RC_MATRIX_VERSION".into(), "0.49.2".into());
    env.insert("KITTY_RC_MATRIX_PROTECTION".into(), "encrypted".into());
    env.insert(
        "KITTY_RC_MATRIX_MARKER".into(),
        "kitty-rc private fixture".into(),
    );
    let dump = parent.path().join("example-env.dump");
    env.insert("FAKE_EXAMPLE_DUMP".into(), dump.display().to_string());

    let arguments = vec![
        fixture.socket.display().to_string(),
        fixture.key.clone(),
        "fixture-test-secret".into(),
    ];
    let output = kitty_rc_verify::examples::run_bounded(
        &bin("example"),
        &arguments,
        &env,
        Duration::from_secs(10),
    )
    .await
    .unwrap();
    assert_eq!(output.code(), Some(0));
    assert!(output.stdout.contains("\"case\":\"fake-example-case\""));

    let dumped = std::fs::read_to_string(&dump).unwrap();
    assert!(
        !dumped.contains("human-key"),
        "ambient public key leaked into the fixture"
    );
    assert!(
        !dumped.contains("human.sock"),
        "ambient listen socket leaked"
    );
    assert!(dumped.contains("WAYLAND_DISPLAY=wl-"));
    assert!(dumped.contains(&format!("XDG_RUNTIME_DIR={}", fixture.dir.display())));

    let dir = fixture.dir.clone();
    fixture.finish().await.unwrap();
    assert!(
        !dir.exists(),
        "clean teardown should remove the private fixture directory"
    );
}

#[tokio::test]
async fn plain_fixture_selects_yes_instead_of_password_mode() {
    let parent = tempfile::tempdir().unwrap();
    let fixture = fixture::start(options(true), parent.path()).await.unwrap();
    let config = std::fs::read_to_string(fixture.dir.join("kitty.conf")).unwrap();
    assert!(config.contains("allow_remote_control yes"));
    assert!(!config.contains("allow_remote_control password"));
    fixture.finish().await.unwrap();
}

#[tokio::test]
async fn failed_server_dependency_is_surfaced_with_log() {
    let parent = tempfile::tempdir().unwrap();
    let mut base = poisoned_base();
    base.insert("FAKE_KITTY_MODE".into(), "fail".into());
    let mut fixture_options = options(false);
    fixture_options.base_env = Some(base);
    let error = fixture::start(fixture_options, parent.path())
        .await
        .unwrap_err();
    match &error {
        FixtureError::ProcessFailed { process, log } => {
            assert_eq!(*process, "kitty");
            assert!(log.contains("configured startup failure"), "log was: {log}");
        }
        other => panic!("expected ProcessFailed, saw {other:?}"),
    }
    assert!(error.to_string().contains("kitty"));
}

#[tokio::test]
async fn startup_deadline_is_bounded() {
    let parent = tempfile::tempdir().unwrap();
    let mut base = poisoned_base();
    base.insert("FAKE_KITTY_MODE".into(), "hang".into());
    let mut fixture_options = options(false);
    fixture_options.base_env = Some(base);
    fixture_options.startup_timeout = Duration::from_millis(400);
    let started = std::time::Instant::now();
    let error = fixture::start(fixture_options, parent.path())
        .await
        .unwrap_err();
    assert!(matches!(error, FixtureError::StartupTimeout { .. }));
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "startup deadline did not bound the wait"
    );
}

fn matrix_options(root: &Path, report: &Path, base: BTreeMap<String, String>) -> MatrixOptions {
    MatrixOptions {
        kitties: vec![bin("kitty")],
        download: false,
        modes: vec![Protection::Encrypted],
        no_build: false,
        finite_only: true,
        cli_smoke: false,
        report: report.to_owned(),
        root: root.to_owned(),
        profile_deadline: Duration::from_secs(60),
        password: "fixture-test-secret".into(),
        weston: bin("weston"),
        fixture_parent: PathBuf::from("/tmp/opencode"),
        target_dir: None,
        base_env: Some(base),
        binaries: Some(Binaries {
            conformance: bin("example"),
            supported_matrix: bin("example"),
            upload_conformance: bin("example"),
            cli: bin("cli"),
        }),
        child: Some(bin("child")),
    }
}

#[tokio::test]
async fn matrix_records_pass_and_writes_evidence() {
    let root = tempfile::tempdir().unwrap();
    let report = root.path().join("matrix.json");
    let options = matrix_options(root.path(), &report, poisoned_base());
    let code = matrix::run(options).await.unwrap();
    assert_eq!(code, 0);
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(report["request_envelope"], serde_json::json!([0, 43, 1]));
    assert_eq!(report["finite_only"], true);
    let server = &report["servers"][0];
    assert_eq!(server["version"], "0.49.2");
    assert_eq!(server["status"], "ready");
    let profile = &server["profiles"][0];
    assert_eq!(profile["protection"], "encrypted");
    assert_eq!(profile["status"], "pass");
    assert!(
        profile["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["case"] == "fake-example-case")
    );
    assert!(report["examples"]["supported_matrix"].is_string());
}

#[tokio::test]
async fn matrix_fails_when_an_example_fails() {
    let root = tempfile::tempdir().unwrap();
    let report = root.path().join("matrix.json");
    let mut base = poisoned_base();
    base.insert("FAKE_EXAMPLE_EXIT".into(), "3".into());
    let options = matrix_options(root.path(), &report, base);
    let code = matrix::run(options).await.unwrap();
    assert_eq!(code, 1);
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let profile = &report["servers"][0]["profiles"][0];
    assert_eq!(profile["status"], "fail");
    let harness = profile["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["harness"] == "supported_matrix")
        .unwrap();
    assert_eq!(harness["returncode"], 3);
    // The fixture password must never appear in recorded output.
    assert!(
        !profile["output"]
            .as_str()
            .unwrap()
            .contains("fixture-test-secret")
    );
}

#[tokio::test]
async fn matrix_blocks_on_missing_executable_without_passing() {
    let root = tempfile::tempdir().unwrap();
    let report = root.path().join("matrix.json");
    let mut options = matrix_options(root.path(), &report, poisoned_base());
    options.kitties = vec![root.path().join("missing/kitty")];
    let code = matrix::run(options).await.unwrap();
    assert_eq!(code, 1);
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(report["servers"][0]["status"], "blocked");
    assert!(
        report["servers"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("not acquired")
    );
}

#[tokio::test]
async fn matrix_deadline_blocks_a_hanging_fixture() {
    let root = tempfile::tempdir().unwrap();
    let report = root.path().join("matrix.json");
    let mut base = poisoned_base();
    base.insert("FAKE_KITTY_MODE".into(), "hang".into());
    let mut options = matrix_options(root.path(), &report, base);
    options.profile_deadline = Duration::from_secs(1);
    let started = std::time::Instant::now();
    let code = matrix::run(options).await.unwrap();
    assert!(started.elapsed() < Duration::from_secs(30));
    assert_eq!(code, 1);
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let profile = &report["servers"][0]["profiles"][0];
    assert_eq!(profile["status"], "blocked");
    assert!(
        profile["output"]
            .as_str()
            .unwrap()
            .contains("bounded deadline")
    );
}

#[tokio::test]
async fn cli_smoke_passes_against_a_cooperative_cli() {
    let record = smoke::run(&SmokeOptions {
        cli: bin("cli"),
        socket: Path::new("/tmp/opencode/kitty-rc-fixture-x/rc.sock").into(),
        key_file: Path::new("/tmp/opencode/kitty-rc-fixture-x/key").into(),
        password: "fixture-test-secret".into(),
        mode: Protection::Encrypted,
        env: poisoned_base(),
        per_query: Duration::from_secs(5),
    })
    .await;
    assert_eq!(record["status"], "pass");
    assert_eq!(record["detail"]["explicit_tab_active"], true);
    assert_eq!(record["detail"]["watch_samples"], 2);
}

#[tokio::test]
async fn cli_smoke_failure_redacts_the_password() {
    let mut env = poisoned_base();
    env.insert("FAKE_CLI_MODE".into(), "fail".into());
    let record = smoke::run(&SmokeOptions {
        cli: bin("cli"),
        socket: Path::new("/tmp/opencode/kitty-rc-fixture-x/rc.sock").into(),
        key_file: Path::new("/tmp/opencode/kitty-rc-fixture-x/key").into(),
        password: "fixture-test-secret".into(),
        mode: Protection::Encrypted,
        env,
        per_query: Duration::from_secs(5),
    })
    .await;
    assert_eq!(record["status"], "fail");
    let text = serde_json::to_string(&record).unwrap();
    assert!(
        !text.contains("fixture-test-secret"),
        "password leaked into smoke failure record"
    );
    assert!(text.contains("<test-password>"));
}

#[tokio::test]
async fn fixture_child_stream_round_trips_binary_payload() {
    // The stream child the upload example launches: exact bytes out, fixed
    // stderr bytes, chosen exit code.
    let payload: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
    let use_stdin = std::process::Stdio::from({
        let mut file = tempfile::tempfile().unwrap();
        use std::io::{Seek as _, Write as _};
        file.write_all(&payload).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file
    });
    let output = tokio::process::Command::new(bin("child"))
        .args(["stream", "--stderr-hex", "00ff80", "--exit", "7"])
        .stdin(use_stdin)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, payload);
    assert_eq!(output.stderr, vec![0x00, 0xff, 0x80]);
}

#[tokio::test]
async fn fixture_child_sentinel_writes_then_exits_with_code() {
    let dir = tempfile::tempdir().unwrap().keep();
    let sentinel = dir.join("started");
    let started = std::time::Instant::now();
    let output = tokio::process::Command::new(bin("child"))
        .args([
            "sentinel",
            sentinel.to_str().unwrap(),
            "--sleep-ms",
            "150",
            "--exit",
            "7",
        ])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "started");
    assert!(started.elapsed() >= Duration::from_millis(120));
}

/// A cold-target run must explicitly request the fixture-child binary:
/// example selection alone never builds sibling verification bins.
#[test]
fn build_plans_always_request_the_fixture_child_binary() {
    for (finite_only, cli_smoke) in [(false, false), (true, false), (false, true), (true, true)] {
        let plans = kitty_rc_verify::examples::matrix_build_plans(finite_only, cli_smoke);
        assert!(
            plans[0]
                .windows(2)
                .any(|window| window == ["--bin", "kitty-rc-verify-child"]),
            "matrix plan missing child bin: {plans:?}"
        );
        assert_eq!(plans.len(), 1 + usize::from(cli_smoke));
        if cli_smoke {
            assert!(plans[1].contains(&"kitty-rc-cli".to_owned()));
        }
        if finite_only {
            assert!(!plans[0].contains(&"conformance".to_owned()));
            assert!(!plans[0].contains(&"upload_conformance".to_owned()));
        } else {
            assert!(plans[0].contains(&"upload_conformance".to_owned()));
        }
    }
    for (package, example) in [
        ("kitty-rc", "conformance"),
        ("kitty-rc-registry", "upload_conformance"),
    ] {
        let plan = kitty_rc_verify::examples::single_example_build_plan(package, example);
        assert!(
            plan.windows(2)
                .any(|window| window == ["--bin", "kitty-rc-verify-child"])
        );
        assert!(plan.contains(&package.to_owned()) && plan.contains(&example.to_owned()));
    }
}

#[tokio::test]
async fn no_build_reports_missing_artifacts_before_any_server() {
    let root = tempfile::tempdir().unwrap();
    let fixtures = tempfile::tempdir().unwrap();
    let cold_target = tempfile::tempdir().unwrap(); // no artifacts inside
    let mut options = matrix_options(
        root.path(),
        &root.path().join("matrix.json"),
        poisoned_base(),
    );
    options.no_build = true;
    options.finite_only = false; // expect the full artifact set to be required
    options.binaries = None;
    options.child = None;
    options.target_dir = Some(cold_target.path().to_owned());
    options.fixture_parent = fixtures.path().to_owned();
    let error = matrix::run(options).await.unwrap_err();
    let missing = match &error {
        matrix::MatrixError::MissingArtifacts { missing, no_build } => {
            assert!(no_build);
            missing.clone()
        }
        other => panic!("expected MissingArtifacts, saw {other:?}"),
    };
    // Every artifact the run needs is named at once, child included.
    for name in ["supported_matrix", "conformance", "upload_conformance"] {
        assert!(
            missing
                .iter()
                .any(|path| path.file_name().is_some_and(|file| file == name)),
            "missing-artifact list lacks {name}: {missing:?}"
        );
    }
    assert!(
        missing.iter().any(|path| path
            .file_name()
            .is_some_and(|file| file == "kitty-rc-verify-child")),
        "missing-artifact list lacks the fixture child: {missing:?}"
    );
    assert!(error.to_string().contains("no server was started"));
    // No fixture directory was created: the blocker fired first.
    assert_eq!(std::fs::read_dir(fixtures.path()).unwrap().count(), 0);
    assert!(!root.path().join("matrix.json").exists());
}

#[tokio::test]
async fn no_build_with_present_artifacts_runs_without_building() {
    let root = tempfile::tempdir().unwrap();
    let fixtures = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let examples = target.path().join("debug/examples");
    std::fs::create_dir_all(&examples).unwrap();
    for name in ["supported_matrix", "conformance", "upload_conformance"] {
        std::fs::copy(bin("example"), examples.join(name)).unwrap();
    }
    std::fs::copy(
        bin("child"),
        target.path().join("debug/kitty-rc-verify-child"),
    )
    .unwrap();
    let report = root.path().join("matrix.json");
    let mut options = matrix_options(root.path(), &report, poisoned_base());
    options.no_build = true;
    options.binaries = None;
    options.child = None;
    options.target_dir = Some(target.path().to_owned());
    options.fixture_parent = fixtures.path().to_owned();
    // The fake kitty serves the fixture and the copied fake-example passes,
    // proving artifact resolution succeeded through the no-build path.
    let code = matrix::run(options).await.unwrap();
    assert_eq!(code, 0);
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(report["servers"][0]["profiles"][0]["status"], "pass");
}
