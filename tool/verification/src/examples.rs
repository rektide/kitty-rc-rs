//! Cargo/mise integration: builds, target discovery and bounded execution of
//! prebuilt example binaries with fully controlled environments.
//!
//! Examples are compiled **before** any server is started (a compiler never
//! runs inside a fixture's bounded lifetime) and executed directly by path;
//! the retired `fixture-cargo.py` PATH adapter has no successor because the
//! Rust runner composes the invocation itself.

use std::collections::BTreeMap;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;

/// Runs cargo through the user's `mise` selection when available; no
/// `+version` override or repository toolchain pin is ever introduced.
#[derive(Clone, Debug)]
pub struct Cargo {
    mise: Option<PathBuf>,
}

impl Cargo {
    pub fn detect() -> Self {
        Self {
            mise: which("mise"),
        }
    }

    fn base(&self) -> (PathBuf, Vec<&'static str>) {
        match &self.mise {
            Some(mise) => (mise.clone(), vec!["exec", "--", "cargo"]),
            None => (PathBuf::from("cargo"), Vec::new()),
        }
    }

    /// The program and argument prefix that run cargo (through the user's
    /// `mise` selection when available) for direct journeys.
    pub fn command_parts(&self) -> (PathBuf, Vec<&'static str>) {
        self.base()
    }

    /// The rustc report recorded in matrix evidence.
    pub async fn rustc_report(&self) -> Result<String, ToolchainError> {
        let (program, args) = match &self.mise {
            Some(mise) => (mise.clone(), vec!["exec", "--", "rustc"]),
            None => (PathBuf::from("rustc"), Vec::new()),
        };
        let output = Command::new(&program)
            .args(args)
            .args(["--version", "--verbose"])
            .output()
            .await
            .map_err(|source| ToolchainError::Probe { source })?;
        if !output.status.success() {
            return Err(ToolchainError::Report(format!(
                "rustc probe exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    /// The workspace target directory from `cargo metadata --no-deps`.
    pub async fn target_directory(&self, root: &Path) -> Result<PathBuf, ToolchainError> {
        let (program, prefix) = self.base();
        let output = Command::new(&program)
            .args(prefix)
            .args(["metadata", "--no-deps", "--format-version=1"])
            .current_dir(root)
            .output()
            .await
            .map_err(|source| ToolchainError::Probe { source })?;
        if !output.status.success() {
            return Err(ToolchainError::Report(format!(
                "cargo metadata exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|error| ToolchainError::Report(format!("cargo metadata parse: {error}")))?;
        let directory = metadata["target_directory"]
            .as_str()
            .ok_or_else(|| ToolchainError::Report("metadata lacks target_directory".into()))?;
        Ok(PathBuf::from(directory))
    }

    /// Run `cargo build --locked …` with inherited stdio so failures surface
    /// verbatim; compilation happens before any fixture starts.
    pub async fn build(&self, root: &Path, args: &[&str]) -> Result<(), ToolchainError> {
        let (program, prefix) = self.base();
        let status = Command::new(&program)
            .args(prefix)
            .args(["build", "--locked"])
            .args(args)
            .current_dir(root)
            .status()
            .await
            .map_err(|source| ToolchainError::Build { source })?;
        if !status.success() {
            return Err(ToolchainError::BuildFailed {
                status: status.to_string(),
            });
        }
        Ok(())
    }
}

/// The cargo target-filter arguments that build the prebuilt binaries one
/// matrix run needs. `--bin kitty-rc-verify-child` is always requested:
/// example selection alone excludes the fixture-child binary, and a cold
/// target must produce it before any server starts.
pub fn matrix_build_plans(finite_only: bool, cli_smoke: bool) -> Vec<Vec<String>> {
    let mut plan = vec![
        "-p".to_owned(),
        "kitty-rc-verify".to_owned(),
        "--bin".to_owned(),
        "kitty-rc-verify-child".to_owned(),
    ];
    if !finite_only {
        plan.extend(["-p".to_owned(), "kitty-rc-registry".to_owned()]);
    }
    plan.extend([
        "-p".to_owned(),
        "kitty-rc".to_owned(),
        "--example".to_owned(),
        "supported_matrix".to_owned(),
    ]);
    if !finite_only {
        plan.extend([
            "--example".to_owned(),
            "conformance".to_owned(),
            "--example".to_owned(),
            "upload_conformance".to_owned(),
        ]);
    }
    let mut plans = vec![plan];
    if cli_smoke {
        plans.push(vec![
            "-p".into(),
            "kitty-rc-cli".into(),
            "--bin".into(),
            "kitty-rc".into(),
        ]);
    }
    plans
}

/// The cargo target-filter arguments building one single-fixture example
/// plus the fixture-child binary it launches. The child is an explicit
/// `--bin` selection for the same cold-target reason.
pub fn single_example_build_plan(package: &str, example: &str) -> Vec<String> {
    vec![
        "-p".into(),
        "kitty-rc-verify".into(),
        "--bin".into(),
        "kitty-rc-verify-child".into(),
        "-p".into(),
        package.into(),
        "--example".into(),
        example.into(),
    ]
}

/// Resolved locations of prebuilt binaries under a cargo target directory.
pub struct ArtifactPaths {
    pub examples: PathBuf,
    pub debug: PathBuf,
}

/// Where a built artifact lives under a cargo target directory.
pub fn artifact_paths(target: &Path) -> ArtifactPaths {
    let debug = target.join("debug");
    ArtifactPaths {
        examples: debug.join("examples"),
        debug,
    }
}

impl ArtifactPaths {
    pub fn example(&self, name: &str) -> PathBuf {
        self.examples.join(name)
    }

    pub fn child(&self) -> PathBuf {
        self.debug.join("kitty-rc-verify-child")
    }

    pub fn cli(&self) -> PathBuf {
        self.debug.join("kitty-rc")
    }
}

fn which(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(program))
        .find(|candidate| candidate.is_file())
}

/// Locate an executable on the current PATH (used for default servers).
pub fn find_on_path(program: &str) -> Option<PathBuf> {
    which(program)
}

#[derive(Debug, Error)]
pub enum ToolchainError {
    #[error("probing the user's toolchain failed")]
    Probe {
        #[source]
        source: std::io::Error,
    },
    #[error("toolchain report: {0}")]
    Report(String),
    #[error("starting cargo build failed")]
    Build {
        #[source]
        source: std::io::Error,
    },
    #[error("cargo build failed with {status}")]
    BuildFailed { status: String },
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("starting {program} failed")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{program} exceeded its bounded deadline of {deadline:?}")]
    Timeout { program: String, deadline: Duration },
    #[error("waiting for {program} failed")]
    Wait {
        program: String,
        #[source]
        source: std::io::Error,
    },
}

/// Captured execution of a prebuilt binary.
#[derive(Debug)]
pub struct RunOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl RunOutput {
    pub fn code(&self) -> Option<i32> {
        self.status.code()
    }
}

/// Execute `program` with a fully replaced environment and bounded lifetime.
/// On timeout the owned process group is killed before the error returns.
pub async fn run_bounded(
    program: &Path,
    args: &[String],
    env: &BTreeMap<String, String>,
    timeout: Duration,
) -> Result<RunOutput, RunError> {
    let mut std_command = std::process::Command::new(program);
    std_command.args(args).env_clear().envs(env);
    std_command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    std_command.process_group(0);
    let mut command = Command::from(std_command);
    command.kill_on_drop(true);
    let spawned = command.spawn().map_err(|source| RunError::Spawn {
        program: program.display().to_string(),
        source,
    })?;
    let program_name = program.display().to_string();
    match tokio::time::timeout(timeout, spawned.wait_with_output()).await {
        Ok(Ok(output)) => Ok(RunOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
        Ok(Err(source)) => Err(RunError::Wait {
            program: program_name,
            source,
        }),
        Err(_) => {
            // kill_on_drop already reaps the direct child; the group backstop
            // in supervision covers tree members.
            Err(RunError::Timeout {
                program: program_name,
                deadline: timeout,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_bounded_propagates_exit_code_and_output() {
        let env = BTreeMap::from([("A".into(), "b".into())]);
        let output = run_bounded(
            Path::new("/bin/sh"),
            &[
                "-c".into(),
                "printf out; printf err 1>&2; test \"$A\" = b && exit 3".into(),
            ],
            &env,
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(output.code(), Some(3));
        assert_eq!(output.stdout, "out");
        assert_eq!(output.stderr, "err");
    }

    #[tokio::test]
    async fn run_bounded_kills_past_deadline_processes() {
        let started = std::time::Instant::now();
        let error = run_bounded(
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 30".into()],
            &BTreeMap::new(),
            Duration::from_millis(300),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, RunError::Timeout { .. }));
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
