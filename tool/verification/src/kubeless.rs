//! Translation of the kubeless comparison probe (design probe, not a
//! shipping dependency). Assembles a disposable comparison package under
//! ignored project scratch from explicit checkout dependencies, then runs
//! the three bounded test journeys. Production Cargo is untouched.

use crate::examples::Cargo;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("assembling the comparison package failed: {context}")]
    Assembly {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("comparison journey {journey} failed with {status}")]
    Journey {
        journey: &'static str,
        status: String,
    },
    #[error("starting journey {journey} failed")]
    Spawn {
        journey: &'static str,
        #[source]
        source: std::io::Error,
    },
}

pub struct ProbeOptions {
    /// The kitty-kubeless checkout providing path dependencies.
    pub kubeless: PathBuf,
    pub root: PathBuf,
}

const DEPENDENCIES: &[(&str, &str, bool)] = &[
    (
        "kubeless-control-retained-actions",
        "control/retained-actions",
        true,
    ),
    (
        "kubeless-control-block-execution",
        "control/block-execution",
        false,
    ),
    ("kubeless-control-plan", "control/plan", false),
    ("kubeless-work-custody", "work/custody", false),
];

/// Run the comparison; returns after all three journeys pass.
pub async fn compare(options: &ProbeOptions) -> Result<(), ProbeError> {
    let kubeless = options
        .kubeless
        .canonicalize()
        .map_err(|source| ProbeError::Assembly {
            context: format!("resolving {}", options.kubeless.display()),
            source,
        })?;
    let scratch = options.root.join(".test-agent/kubeless-comparison");
    std::fs::create_dir_all(&scratch).map_err(|source| ProbeError::Assembly {
        context: "creating scratch".into(),
        source,
    })?;
    std::fs::write(
        scratch.join("README.md"),
        "Generated comparison-only package. Its explicit checkout dependencies are not a portable \
         or published kitty dependency. Source probes and runner live under \
         tool/verification (kubeless-compare subcommand).\n",
    )
    .map_err(|source| ProbeError::Assembly {
        context: "writing README".into(),
        source,
    })?;

    let mut manifest = String::from(
        "[workspace]\n[package]\nname=\"kitty-kubeless-comparison\"\nversion=\"0.0.0\"\nedition=\"2024\"\n\
         [features]\ndefault=[\"host\"]\nhost=[]\ntimed=[]\n[dependencies]\n",
    );
    for (name, relative, host) in DEPENDENCIES {
        let features = if *host { ", features=[\"host\"]" } else { "" };
        manifest.push_str(&format!(
            "{name}={{path=\"{}\"{features}}}\n",
            kubeless.join(relative).display()
        ));
    }
    manifest.push_str("\n[[test]]\nname=\"direct\"\npath=\"direct.rs\"\n[[test]]\nname=\"host\"\npath=\"host.rs\"\n");
    std::fs::write(scratch.join("Cargo.toml"), &manifest).map_err(|source| {
        ProbeError::Assembly {
            context: "writing manifest".into(),
            source,
        }
    })?;

    let direct = std::fs::read_to_string(
        options
            .root
            .join("design/library-redesign/probes/direct-journeys0.sol61m.rs"),
    )
    .map_err(|source| ProbeError::Assembly {
        context: "reading direct journeys".into(),
        source,
    })?;
    std::fs::write(scratch.join("direct.rs"), direct).map_err(|source| ProbeError::Assembly {
        context: "writing direct.rs".into(),
        source,
    })?;
    let upstream_host = std::fs::read_to_string(
        kubeless.join("control/retained-actions/tests/host.rs"),
    )
    .map_err(|source| ProbeError::Assembly {
        context: "reading upstream host tests".into(),
        source,
    })?;
    let probe_host = std::fs::read_to_string(
        options
            .root
            .join("design/library-redesign/probes/host-journeys0.sol61m.rs"),
    )
    .map_err(|source| ProbeError::Assembly {
        context: "reading host probe".into(),
        source,
    })?;
    std::fs::write(
        scratch.join("host.rs"),
        format!("{upstream_host}\n{probe_host}"),
    )
    .map_err(|source| ProbeError::Assembly {
        context: "writing host.rs".into(),
        source,
    })?;

    let cargo = Cargo::detect();
    let (program, prefix) = cargo.command_parts();
    let comparison_arguments = vec![
        "test".to_owned(),
        "--manifest-path".to_owned(),
        scratch.join("Cargo.toml").display().to_string(),
    ];
    run_journey(
        &program,
        &prefix,
        "comparison package",
        comparison_arguments,
        None,
    )
    .await?;
    run_journey(
        &program,
        &prefix,
        "upstream attached custody",
        vec![
            "test".to_owned(),
            "-p".to_owned(),
            "kubeless-work-custody".to_owned(),
            "--test".to_owned(),
            "attached".to_owned(),
        ],
        Some(&kubeless),
    )
    .await?;
    run_journey(
        &program,
        &prefix,
        "registry journeys",
        vec![
            "test".to_owned(),
            "-p".to_owned(),
            "kitty-rc-registry".to_owned(),
            "--test".to_owned(),
            "journeys".to_owned(),
        ],
        Some(&options.root),
    )
    .await?;
    Ok(())
}

async fn run_journey(
    program: &Path,
    prefix: &[&'static str],
    journey: &'static str,
    arguments: Vec<String>,
    cwd: Option<&Path>,
) -> Result<(), ProbeError> {
    let mut command = Command::new(program);
    command.args(prefix).args(arguments).stdin(Stdio::null());
    if let Some(directory) = cwd {
        command.current_dir(directory);
    }
    let status = command
        .status()
        .await
        .map_err(|source| ProbeError::Spawn { journey, source })?;
    if !status.success() {
        return Err(ProbeError::Journey {
            journey,
            status: status.to_string(),
        });
    }
    Ok(())
}
