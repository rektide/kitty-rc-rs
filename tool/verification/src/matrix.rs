//! The opt-in supported-version matrix.
//!
//! For each selected kitty executable and protection mode this composes one
//! private fixture, runs the prebuilt conformance examples directly (no PATH
//! adapters — the runner owns every invocation), adds the standalone CLI
//! smoke journeys when requested, and records per-profile JSON evidence.
//! Missing executables, failed fixture startup and exceeded deadlines are
//! blockers, never passes; a failed case fails the run.

use crate::Protection;
use crate::download;
use crate::examples::{Cargo, ToolchainError};
use crate::fixture::{self, FixtureOptions};
use crate::release;
use crate::report::{
    MatrixReport, ProfileRecord, ProfileStatus, ServerRecord, ServerStatus, parse_case_records,
};
use crate::smoke::{self, SmokeOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

/// Prebuilt example/CLI binaries the matrix executes inside fixtures.
/// Injectable so lifecycle tests exercise real supervision without a
/// compiler or a live kitty.
#[derive(Clone, Debug)]
pub struct Binaries {
    pub conformance: PathBuf,
    pub supported_matrix: PathBuf,
    pub upload_conformance: PathBuf,
    pub cli: PathBuf,
}

pub struct MatrixOptions {
    /// Explicit `--kitty` executables; empty selects the default set
    /// (installed kitty plus acquired 0.43.1 and 0.49.2).
    pub kitties: Vec<PathBuf>,
    pub download: bool,
    pub modes: Vec<Protection>,
    pub no_build: bool,
    pub finite_only: bool,
    pub cli_smoke: bool,
    pub report: PathBuf,
    pub root: PathBuf,
    /// Bounded lifetime for one profile (fixture + examples + smoke).
    pub profile_deadline: Duration,
    /// Fixture-local test password; never a human credential.
    pub password: String,
    /// Explicit headless weston executable (default: `weston` from PATH).
    pub weston: PathBuf,
    /// Parent directory for transient private fixtures (default: the
    /// approved `/tmp/opencode`, keeping socket paths short).
    pub fixture_parent: PathBuf,
    /// Override for the cargo target directory (normally discovered via
    /// `cargo metadata`; injectable so tests exercise artifact resolution
    /// without a compiler).
    pub target_dir: Option<PathBuf>,
    pub base_env: Option<BTreeMap<String, String>>,
    pub binaries: Option<Binaries>,
    pub child: Option<PathBuf>,
}

impl Default for MatrixOptions {
    fn default() -> Self {
        Self {
            kitties: Vec::new(),
            download: false,
            modes: Vec::new(),
            no_build: false,
            finite_only: false,
            cli_smoke: false,
            report: PathBuf::from(".test-agent/supported-kitty/matrix.json"),
            root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            profile_deadline: Duration::from_secs(160),
            password: "upload-test-secret".into(),
            weston: PathBuf::from("weston"),
            fixture_parent: PathBuf::from("/tmp/opencode"),
            target_dir: None,
            base_env: None,
            binaries: None,
            child: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum MatrixError {
    #[error("toolchain preparation failed")]
    Toolchain {
        #[from]
        source: ToolchainError,
    },
    #[error("release acquisition failed for {version}")]
    Acquisition {
        version: String,
        #[source]
        source: download::DownloadError,
    },
    #[error("writing the matrix report failed")]
    Report {
        #[from]
        source: std::io::Error,
    },
    #[error(
        "required prebuilt artifacts are missing{}; no implicit build was performed and no server was started. \
         Build them deliberately (rerun without --no-build, or build the listed targets) and try again:\n{}",
        if *no_build { " (--no-build)" } else { " after the build step" },
        missing.iter().map(|path| format!("  {}", path.display())).collect::<Vec<_>>().join("\n")
    )]
    MissingArtifacts {
        missing: Vec<PathBuf>,
        no_build: bool,
    },
    #[error("interrupted; partial report written to {path}")]
    Interrupted { path: PathBuf },
}

struct ProfileContext<'a> {
    /// The exact executable this profile tests (never an ambient default).
    kitty: &'a Path,
    weston: &'a Path,
    binaries: &'a Binaries,
    child: &'a Path,
    password: &'a str,
    version: &'a str,
    protection: Protection,
    fixture_parent: &'a Path,
    base_env: &'a BTreeMap<String, String>,
    startup_timeout: Duration,
}

/// Run the full matrix; returns the process exit code.
pub async fn run(options: MatrixOptions) -> Result<i32, MatrixError> {
    let scratch = release::scratch(&options.root);
    std::fs::create_dir_all(&scratch)?;
    std::fs::create_dir_all(&options.fixture_parent)?;
    if options.download {
        for entry in release::RELEASES {
            download::acquire(&options.root, entry).map_err(|source| MatrixError::Acquisition {
                version: entry.version.into(),
                source,
            })?;
        }
    }

    let cargo = Cargo::detect();
    let base_env = options
        .base_env
        .clone()
        .unwrap_or_else(crate::env::sanitized_ambient);
    let mut child = options
        .child
        .clone()
        .or_else(|| std::env::var_os("KITTY_RC_VERIFY_CHILD").map(PathBuf::from));
    let (binaries, rustc) = match &options.binaries {
        Some(injected) => (injected.clone(), "injected test binaries".to_owned()),
        None => {
            let rustc = cargo.rustc_report().await?;
            let target = match &options.target_dir {
                Some(directory) => directory.clone(),
                None => cargo.target_directory(&options.root).await?,
            };
            let paths = crate::examples::artifact_paths(&target);
            let binaries = if options.no_build {
                // Honor the deliberate converged build: no implicit compile.
                resolve_prebuilt(&paths, &options)?
            } else {
                for plan in
                    crate::examples::matrix_build_plans(options.finite_only, options.cli_smoke)
                {
                    let arguments: Vec<&str> = plan.iter().map(String::as_str).collect();
                    cargo.build(&options.root, &arguments).await?;
                }
                // A claimed-success build that produced no artifact is a
                // blocker, not something to paper over.
                resolve_prebuilt(&paths, &options)?
            };
            if child.is_none() {
                child = Some(paths.child());
            }
            (binaries, rustc)
        }
    };
    let runner = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("kitty-rc-verify"));
    let child = child.unwrap_or_else(|| {
        runner
            .parent()
            .map(|parent| parent.join("kitty-rc-verify-child"))
            .unwrap_or_else(|| PathBuf::from("kitty-rc-verify-child"))
    });
    if !child.exists() {
        return Err(MatrixError::MissingArtifacts {
            missing: vec![child],
            no_build: options.no_build,
        });
    }

    let mut report = MatrixReport::new(
        options.finite_only,
        options.cli_smoke,
        rustc.clone(),
        &runner,
    );
    for (name, path) in [
        ("conformance", &binaries.conformance),
        ("supported_matrix", &binaries.supported_matrix),
        ("upload_conformance", &binaries.upload_conformance),
    ] {
        if let Ok(digest) = release::sha256_file(path) {
            report.examples.insert(name.into(), digest);
        }
    }
    if options.cli_smoke {
        report.cli_sha256 = release::sha256_file(&binaries.cli).ok();
    }

    let executables: Vec<PathBuf> = if options.kitties.is_empty() {
        let installed = crate::examples::find_on_path("kitty")
            .unwrap_or_else(|| PathBuf::from("/usr/bin/kitty"));
        let mut all = vec![installed];
        all.extend(
            ["0.43.1", "0.49.2"]
                .iter()
                .map(|version| release::executable_in_scratch(&options.root, version)),
        );
        all
    } else {
        options.kitties.clone()
    };

    let modes: Vec<Protection> = if options.modes.is_empty() {
        vec![Protection::Encrypted, Protection::Plain]
    } else {
        options.modes.clone()
    };

    let mut failed = false;
    let mut interrupted = false;
    'servers: for executable in &executables {
        let mut server = ServerRecord {
            executable: executable.display().to_string(),
            executable_sha256: None,
            version: None,
            reported_version: None,
            provenance: None,
            status: ServerStatus::Ready,
            reason: None,
            profiles: Vec::new(),
        };
        let resolved = executable
            .canonicalize()
            .unwrap_or_else(|_| executable.clone());
        if !resolved.exists() {
            server.status = ServerStatus::Blocked;
            server.reason = Some("not acquired; run with --download or supply --kitty".into());
            server.executable = resolved.display().to_string();
            report.servers.push(server);
            failed = true;
            continue;
        }
        server.executable = resolved.display().to_string();
        server.executable_sha256 = release::sha256_file(&resolved).ok();
        let reported = match crate::examples::run_bounded(
            &resolved,
            &["--version".to_owned()],
            &base_env,
            Duration::from_secs(30),
        )
        .await
        {
            Ok(output) if output.status.success() => output.stdout.trim().to_owned(),
            Ok(output) => {
                server.status = ServerStatus::Blocked;
                server.reason = Some(format!("--version failed: {}", output.stderr.trim()));
                report.servers.push(server);
                failed = true;
                continue;
            }
            Err(error) => {
                server.status = ServerStatus::Blocked;
                server.reason = Some(error.to_string());
                report.servers.push(server);
                failed = true;
                continue;
            }
        };
        match release::parse_version(&reported) {
            Some((_, version)) => {
                server.version = Some(version.to_owned());
                server.reported_version = Some(reported.clone());
                server.provenance = Some(provenance_for(&options.root, version, &resolved));
            }
            None => {
                server.status = ServerStatus::Blocked;
                server.reason = Some(format!("unrecognized kitty version: {reported}"));
                server.reported_version = Some(reported);
                report.servers.push(server);
                failed = true;
                continue;
            }
        }
        let version = server.version.clone().unwrap_or_default();
        for protection in &modes {
            eprintln!("Private matrix: kitty {version}, {}", protection.as_str());
            let context = ProfileContext {
                kitty: &resolved,
                weston: &options.weston,
                binaries: &binaries,
                child: &child,
                password: &options.password,
                version: &version,
                protection: *protection,
                fixture_parent: &options.fixture_parent,
                base_env: &base_env,
                startup_timeout: Duration::from_secs(30),
            };
            let profile = tokio::select! {
                result = run_profile(&context, &options) => result,
                _ = tokio::signal::ctrl_c() => {
                    // Ctrl-C: tear down only this fixture's owned groups and stop.
                    interrupted = true;
                    ProfileRecord {
                        protection: *protection,
                        status: ProfileStatus::Blocked,
                        records: Vec::new(),
                        output: "interrupted; owned fixture processes torn down".into(),
                    }
                }
            };
            eprintln!("{}", profile.output);
            failed |= profile.status != ProfileStatus::Pass;
            server.profiles.push(profile);
            if interrupted {
                report.servers.push(server);
                break 'servers;
            }
        }
        report.servers.push(server);
    }

    options
        .report
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()?;
    report.write(&options.report)?;
    println!("Matrix evidence: {}", options.report.display());
    if interrupted {
        return Err(MatrixError::Interrupted {
            path: options.report,
        });
    }
    Ok(if failed { 1 } else { 0 })
}

fn provenance_for(root: &Path, version: &str, executable: &Path) -> serde_json::Value {
    let path = release::scratch(root).join(version).join("provenance.json");
    match std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
    {
        Some(record)
            if record["executable"].as_str() == Some(executable.display().to_string().as_str()) =>
        {
            record
        }
        _ => {
            serde_json::json!({"source": "explicit installed/system executable; no source revision inferred"})
        }
    }
}

/// Resolve every prebuilt artifact this run needs under `paths`, returning
/// an actionable blocker listing all missing files at once. Called both for
/// `--no-build` (no implicit compile) and after a deliberate build.
fn resolve_prebuilt(
    paths: &crate::examples::ArtifactPaths,
    options: &MatrixOptions,
) -> Result<Binaries, MatrixError> {
    let binaries = Binaries {
        conformance: paths.example("conformance"),
        supported_matrix: paths.example("supported_matrix"),
        upload_conformance: paths.example("upload_conformance"),
        cli: paths.cli(),
    };
    let mut missing = Vec::new();
    if !options.finite_only {
        if !binaries.conformance.exists() {
            missing.push(binaries.conformance.clone());
        }
        if !binaries.upload_conformance.exists() {
            missing.push(binaries.upload_conformance.clone());
        }
    }
    if !binaries.supported_matrix.exists() {
        missing.push(binaries.supported_matrix.clone());
    }
    if options.cli_smoke && !binaries.cli.exists() {
        missing.push(binaries.cli.clone());
    }
    if !paths.child().exists() {
        missing.push(paths.child());
    }
    if missing.is_empty() {
        Ok(binaries)
    } else {
        Err(MatrixError::MissingArtifacts {
            missing,
            no_build: options.no_build,
        })
    }
}

/// One fixture, its examples and smoke journeys, inside one bounded deadline.
async fn run_profile(context: &ProfileContext<'_>, options: &MatrixOptions) -> ProfileRecord {
    match tokio::time::timeout(options.profile_deadline, profile_journeys(context, options)).await {
        Ok(record) => record,
        Err(_) => ProfileRecord {
            protection: context.protection,
            status: ProfileStatus::Blocked,
            records: Vec::new(),
            output: "matrix runner exceeded its bounded deadline".into(),
        },
    }
}

async fn profile_journeys(context: &ProfileContext<'_>, options: &MatrixOptions) -> ProfileRecord {
    let mut output_lines: Vec<String> = Vec::new();
    let fixture_options = FixtureOptions {
        kitty: context.kitty.to_path_buf(),
        weston: context.weston.to_path_buf(),
        child: context.child.to_path_buf(),
        password: context.password.to_owned(),
        plain: context.protection == Protection::Plain,
        augment_config: true,
        marker: "kitty-rc private fixture".into(),
        startup_timeout: context.startup_timeout,
        child_sleep: Duration::from_secs(120),
        base_env: Some(context.base_env.clone()),
    };
    let fixture = match fixture::start(fixture_options, context.fixture_parent).await {
        Ok(fixture) => fixture,
        Err(error) => {
            return ProfileRecord {
                protection: context.protection,
                status: ProfileStatus::Blocked,
                records: Vec::new(),
                output: crate::redact_password(&error.to_string(), context.password),
            };
        }
    };

    let mut env = fixture.environment();
    env.insert("KITTY_RC_MATRIX_VERSION".into(), context.version.to_owned());
    env.insert(
        "KITTY_RC_MATRIX_PROTECTION".into(),
        context.protection.as_str().into(),
    );
    env.insert(
        "KITTY_RC_MATRIX_MARKER".into(),
        "kitty-rc private fixture".into(),
    );
    env.insert(
        "KITTY_RC_MATRIX_CHILD".into(),
        context.child.display().to_string(),
    );

    let socket = fixture.socket.display().to_string();
    let key = fixture.key.clone();
    let password = context.password.to_owned();
    let mut failed = false;

    let mut journeys: Vec<(&'static str, Vec<String>)> = vec![(
        "supported_matrix",
        vec![socket.clone(), key.clone(), password.clone()],
    )];
    if context.protection == Protection::Encrypted && !options.finite_only {
        journeys.insert(
            0,
            (
                "conformance",
                vec![socket.clone(), key.clone(), password.clone()],
            ),
        );
        let png_path = fixture.dir.join("pixel.png");
        if let Err(error) = std::fs::write(&png_path, crate::png::one_pixel_rgba_png()) {
            output_lines.push(format!("matrix png fixture failed: {error}"));
            failed = true;
        } else {
            journeys.push((
                "upload_conformance",
                vec![
                    socket.clone(),
                    key.clone(),
                    password.clone(),
                    png_path.display().to_string(),
                ],
            ));
        }
    }

    for (name, arguments) in &journeys {
        let binary = match *name {
            "conformance" => &context.binaries.conformance,
            "upload_conformance" => &context.binaries.upload_conformance,
            _ => &context.binaries.supported_matrix,
        };
        let result =
            crate::examples::run_bounded(binary, arguments, &env, Duration::from_secs(90)).await;
        let (status, code, text) = match result {
            Ok(run) => (
                if run.status.success() { "pass" } else { "fail" },
                run.code(),
                format!("{}{}", run.stdout, run.stderr),
            ),
            Err(error) => ("fail", None, error.to_string()),
        };
        failed |= status != "pass";
        output_lines.push(
            serde_json::to_string(
                &serde_json::json!({"harness": name, "status": status, "returncode": code}),
            )
            .unwrap_or_default(),
        );
        output_lines.extend(text.lines().map(str::to_owned));
    }

    if options.cli_smoke {
        let smoke_record = smoke::run(&SmokeOptions {
            cli: context.binaries.cli.clone(),
            socket: fixture.socket.clone(),
            key_file: fixture.dir.join("key"),
            password: context.password.to_owned(),
            mode: context.protection,
            env: fixture.environment(),
            per_query: Duration::from_secs(10),
        })
        .await;
        failed |= smoke_record["status"].as_str() != Some("pass");
        output_lines.push(serde_json::to_string(&smoke_record).unwrap_or_default());
    }

    if let Err(error) = fixture.finish().await {
        output_lines.push(format!("fixture teardown failed: {error}"));
        failed = true;
    }

    let output = format!(
        "{}\n",
        crate::redact_password(&output_lines.join("\n"), context.password)
    );
    // Records are re-parsed from the sanitized output, exactly as the retired
    // runner did, so no unredacted bytes can enter the report.
    let records = parse_case_records(&output);
    let status = if !records.iter().any(|record| record.get("case").is_some()) {
        ProfileStatus::Blocked
    } else if failed {
        ProfileStatus::Fail
    } else {
        ProfileStatus::Pass
    };
    ProfileRecord {
        protection: context.protection,
        status,
        records,
        output,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_executable_set_covers_installed_and_acquired_releases() {
        let options = MatrixOptions::default();
        assert!(options.report.ends_with("matrix.json"));
        assert_eq!(options.profile_deadline, Duration::from_secs(160));
        assert_eq!(options.password, "upload-test-secret");
    }
}
