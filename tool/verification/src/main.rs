//! `kitty-rc-verify` — the private Rust verification entrance for the
//! kitty-rc workspace. See `tool/verification/README.md` for the full guide.

use clap::{Parser, Subcommand};
use kitty_rc_verify::{
    Protection, download, examples::Cargo, fixture, kubeless, matrix, smoke, source,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "kitty-rc-verify",
    about = "Private Rust verification orchestration for kitty-rc",
    long_about = "Isolated headless fixtures, supported-version matrix, pinned release \
                  acquisition, standalone CLI smoke and pinned source oracles. Never \
                  selects an ambient human terminal, key or config."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Private headless finite conformance against one explicit kitty.
    Finite(FiniteArgs),
    /// Private headless upload conformance (PNG/logo/run journeys).
    Uploads(UploadsArgs),
    /// Opt-in supported-version matrix over selected executables and modes.
    Matrix(MatrixArgs),
    /// Acquire pinned upstream release binaries into ignored scratch.
    Download(DownloadArgs),
    /// Standalone CLI smoke journeys against one existing private fixture.
    CliSmoke(CliSmokeArgs),
    /// Verify pinned upstream source contracts through the minimal oracle.
    Source(SourceArgs),
    /// Compare against a kubeless checkout (design probe, scratch only).
    KubelessCompare(KubelessArgs),
}

/// Arguments shared by the single-fixture runners.
#[derive(clap::Args)]
struct FixtureArgs {
    /// Explicit kitty executable; defaults to $KITTY_RC_TEST_KITTY, then PATH.
    #[arg(long)]
    kitty: Option<PathBuf>,
    /// Explicit headless weston executable.
    #[arg(long, default_value = "weston")]
    weston: PathBuf,
    /// Parent directory for the private fixture (created if needed).
    #[arg(long, default_value = "/tmp/opencode")]
    scratch: PathBuf,
    /// Fixture-local test password; never a human credential.
    #[arg(long)]
    password: Option<String>,
    /// Bounded fixture startup deadline in seconds.
    #[arg(long, default_value_t = 30)]
    startup_timeout: u64,
}

impl FixtureArgs {
    fn kitty(&self) -> PathBuf {
        self.kitty
            .clone()
            .or_else(|| std::env::var_os("KITTY_RC_TEST_KITTY").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("kitty"))
    }
}

#[derive(clap::Args)]
struct FiniteArgs {
    #[command(flatten)]
    fixture: FixtureArgs,
}

#[derive(clap::Args)]
struct UploadsArgs {
    #[command(flatten)]
    fixture: FixtureArgs,
}

#[derive(clap::Args)]
struct MatrixArgs {
    /// Explicit executable (repeatable); default: installed plus acquired.
    #[arg(long = "kitty")]
    kitties: Vec<PathBuf>,
    /// Acquire pinned 0.43.1 and 0.49.2 first (explicit network opt-in).
    #[arg(long)]
    download: bool,
    /// Protection mode (repeatable); defaults to both.
    #[arg(long = "mode")]
    modes: Vec<Protection>,
    /// Use prebuilt artifacts only; every required example, the CLI binary
    /// (with --cli-smoke) and the fixture child must already exist. Missing
    /// artifacts are reported as a blocker before any server starts; no
    /// implicit build is performed.
    #[arg(long)]
    no_build: bool,
    /// Preparation/finite profiles only; omit core and upload journeys.
    #[arg(long)]
    finite_only: bool,
    /// Also exercise standalone CLI journeys with the 16 MiB budget.
    #[arg(long)]
    cli_smoke: bool,
    /// JSON report path.
    #[arg(long, default_value = ".test-agent/supported-kitty/matrix.json")]
    report: PathBuf,
    /// Bounded lifetime per profile in seconds.
    #[arg(long, default_value_t = 160)]
    deadline: u64,
    /// Fixture-local test password; never a human credential.
    #[arg(long, default_value = "upload-test-secret")]
    password: String,
}

#[derive(clap::Args)]
struct DownloadArgs {
    /// Versions to acquire (default: every pinned release).
    versions: Vec<String>,
}

#[derive(clap::Args)]
struct CliSmokeArgs {
    /// The private fixture's RC socket path.
    socket: PathBuf,
    /// The fixture's published public key (encrypted mode only).
    key: Option<String>,
    /// The fixture-local test password.
    password: String,
    /// Protection mode.
    #[arg(long, default_value = "encrypted")]
    auth: Protection,
    /// Explicit kitty-rc CLI binary (default: workspace build).
    #[arg(long)]
    cli: Option<PathBuf>,
    /// Key file to pass instead of --key-file next to the socket.
    #[arg(long)]
    key_file: Option<PathBuf>,
}

#[derive(clap::Args)]
struct SourceArgs {
    /// Path to a kovidgoyal/kitty checkout readable by git.
    checkout: PathBuf,
    /// Explicit python3 for the minimal oracle boundary.
    #[arg(long, default_value = "python3")]
    python: PathBuf,
}

#[derive(clap::Args)]
struct KubelessArgs {
    /// Path to the kitty-kubeless checkout.
    kubeless: PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let root = std::env::current_dir().expect("current directory");
    let code = match cli.command {
        Command::Finite(args) => run_single_example(args.fixture, "conformance", false, 5).await,
        Command::Uploads(args) => {
            run_single_example(args.fixture, "upload_conformance", true, 90).await
        }
        Command::Matrix(args) => match matrix::run(matrix_options(&args, &root)).await {
            Ok(code) => code,
            Err(error) => {
                eprintln!("{error}");
                match error {
                    matrix::MatrixError::Interrupted { .. } => 130,
                    _ => 2,
                }
            }
        },
        Command::Download(args) => {
            let versions: Vec<String> = if args.versions.is_empty() {
                kitty_rc_verify::release::RELEASES
                    .iter()
                    .map(|entry| entry.version.into())
                    .collect()
            } else {
                args.versions
            };
            let mut failed = false;
            for version in versions {
                match kitty_rc_verify::release::release(&version) {
                    Some(entry) => match download::acquire(&root, entry) {
                        Ok(provenance) => println!("{provenance}"),
                        Err(error) => {
                            eprintln!("{error}");
                            failed = true;
                        }
                    },
                    None => {
                        eprintln!("no pinned release {version}");
                        failed = true;
                    }
                }
            }
            if failed { 1 } else { 0 }
        }
        Command::CliSmoke(args) => cli_smoke(&args, &root).await,
        Command::Source(args) => {
            let options = source::SourceOptions {
                checkout: args.checkout,
                root: root.clone(),
                python: args.python,
            };
            match source::verify(&options) {
                Ok(summary) => {
                    println!("{summary}");
                    0
                }
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        Command::KubelessCompare(args) => {
            let options = kubeless::ProbeOptions {
                kubeless: args.kubeless,
                root: root.clone(),
            };
            match kubeless::compare(&options).await {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
    };
    std::process::exit(code);
}

fn matrix_options(args: &MatrixArgs, root: &std::path::Path) -> matrix::MatrixOptions {
    matrix::MatrixOptions {
        kitties: args.kitties.clone(),
        download: args.download,
        modes: args.modes.clone(),
        no_build: args.no_build,
        finite_only: args.finite_only,
        cli_smoke: args.cli_smoke,
        report: args.report.clone(),
        root: root.to_owned(),
        profile_deadline: Duration::from_secs(args.deadline),
        password: args.password.clone(),
        weston: PathBuf::from("weston"),
        fixture_parent: PathBuf::from("/tmp/opencode"),
        target_dir: None,
        base_env: None,
        binaries: None,
        child: None,
    }
}

/// The single-example build invocation as static strings (the two callers
/// are fixed, so the mapping is total). Kept in lockstep with
/// [`kitty_rc_verify::examples::single_example_build_plan`], which tests
/// assert against.
fn single_example_build_arguments(package: &str, example: &str) -> Vec<&'static str> {
    let package = match package {
        "kitty-rc" => "kitty-rc",
        _ => "kitty-rc-registry",
    };
    let example = match example {
        "conformance" => "conformance",
        _ => "upload_conformance",
    };
    vec![
        "-p",
        "kitty-rc-verify",
        "--bin",
        "kitty-rc-verify-child",
        "-p",
        package,
        "--example",
        example,
    ]
}

/// One private fixture, one example, streaming output live.
async fn run_single_example(
    args: FixtureArgs,
    example: &'static str,
    needs_png: bool,
    example_timeout: u64,
) -> i32 {
    let password = args.password.clone().unwrap_or_else(|| {
        if example == "conformance" {
            "test-secret".into()
        } else {
            "upload-test-secret".into()
        }
    });
    let scratch = args.scratch.clone();
    if let Err(error) = std::fs::create_dir_all(&scratch) {
        eprintln!("cannot create scratch {}: {error}", scratch.display());
        return 2;
    }
    let cargo = Cargo::detect();
    let target = match cargo.target_directory(std::path::Path::new(".")).await {
        Ok(target) => target,
        Err(error) => {
            eprintln!("{error}");
            return 2;
        }
    };
    let package = if example == "conformance" {
        "kitty-rc"
    } else {
        "kitty-rc-registry"
    };
    if let Err(error) = cargo
        .build(
            std::path::Path::new("."),
            &single_example_build_arguments(package, example),
        )
        .await
    {
        eprintln!("{error}");
        return 2;
    }
    // Resolve the fixture child from the actual cargo target directory so a
    // cold target (or a redirected CARGO_TARGET_DIR) works; the plan built
    // it explicitly as a --bin selection.
    let artifacts = kitty_rc_verify::examples::artifact_paths(&target);
    let child = std::env::var_os("KITTY_RC_VERIFY_CHILD")
        .map(PathBuf::from)
        .unwrap_or_else(|| artifacts.child());
    if !child.exists() {
        eprintln!(
            "fixture child {} is missing after the build step; refusing to start a server",
            child.display()
        );
        return 2;
    }
    let binary = artifacts.example(example);

    let fixture_args = fixture::FixtureOptions {
        kitty: args.kitty(),
        weston: args.weston.clone(),
        child: child.clone(),
        password: password.clone(),
        plain: false,
        augment_config: false,
        marker: "kitty-rc private fixture".into(),
        startup_timeout: Duration::from_secs(args.startup_timeout),
        child_sleep: Duration::from_secs(120),
        base_env: None,
    };
    let fixture = match fixture::start(fixture_args, &scratch).await {
        Ok(fixture) => fixture,
        Err(error) => {
            eprintln!(
                "{}",
                kitty_rc_verify::redact_password(&error.to_string(), &password)
            );
            return 1;
        }
    };
    println!(
        "{}",
        kitty_rc_verify::release::sha256_file(&args.kitty())
            .map(|digest| format!("private kitty executable sha256: {digest}"))
            .unwrap_or_default()
    );
    let mut arguments = vec![
        fixture.socket.display().to_string(),
        fixture.key.clone(),
        password.clone(),
    ];
    if needs_png {
        let png = fixture.dir.join("pixel.png");
        if let Err(error) = std::fs::write(&png, kitty_rc_verify::png::one_pixel_rgba_png()) {
            eprintln!("png fixture failed: {error}");
            let _ = fixture.finish().await;
            return 2;
        }
        arguments.push(png.display().to_string());
    }
    let mut env = fixture.environment();
    env.insert("KITTY_RC_MATRIX_CHILD".into(), child.display().to_string());
    let result = kitty_rc_verify::examples::run_bounded(
        &binary,
        &arguments,
        &env,
        Duration::from_secs(example_timeout),
    )
    .await;
    let code = match result {
        Ok(output) => {
            print!(
                "{}",
                kitty_rc_verify::redact_password(&output.stdout, &password)
            );
            eprint!(
                "{}",
                kitty_rc_verify::redact_password(&output.stderr, &password)
            );
            output.code().unwrap_or(1)
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    };
    if let Err(error) = fixture.finish().await {
        eprintln!("teardown failed: {error}");
        return 1;
    }
    code
}

async fn cli_smoke(args: &CliSmokeArgs, root: &std::path::Path) -> i32 {
    let cli = match &args.cli {
        Some(path) => path.clone(),
        None => {
            let cargo = Cargo::detect();
            let target = match cargo.target_directory(root).await {
                Ok(target) => target,
                Err(error) => {
                    eprintln!("{error}");
                    return 2;
                }
            };
            if let Err(error) = cargo
                .build(root, &["-p", "kitty-rc-cli", "--bin", "kitty-rc"])
                .await
            {
                eprintln!("{error}");
                return 2;
            }
            target.join("debug/kitty-rc")
        }
    };
    let key_file = args
        .key_file
        .clone()
        .unwrap_or_else(|| args.socket.parent().unwrap().join("key"));
    if let Some(key) = &args.key
        && let Err(error) = std::fs::write(&key_file, key)
    {
        eprintln!("cannot stage key file {}: {error}", key_file.display());
        return 2;
    }
    let record = smoke::run(&smoke::SmokeOptions {
        cli,
        socket: args.socket.clone(),
        key_file,
        password: args.password.clone(),
        mode: args.auth,
        env: kitty_rc_verify::env::sanitized_ambient(),
        per_query: Duration::from_secs(10),
    })
    .await;
    println!("{}", serde_json::to_string_pretty(&record).unwrap());
    if record["status"].as_str() == Some("pass") {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_example_arguments_match_the_lib_build_plan() {
        for (package, example) in [
            ("kitty-rc", "conformance"),
            ("kitty-rc-registry", "upload_conformance"),
        ] {
            let arguments = single_example_build_arguments(package, example);
            let plan = kitty_rc_verify::examples::single_example_build_plan(package, example);
            assert_eq!(
                arguments,
                plan.iter().map(String::as_str).collect::<Vec<_>>(),
                "main.rs build invocation drifted from the tested plan"
            );
        }
    }
}
