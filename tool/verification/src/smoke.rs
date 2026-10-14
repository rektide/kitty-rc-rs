//! Standalone CLI smoke journeys against one existing private fixture.
//!
//! Exercises the real `kitty-rc` binary exactly as a user would — list,
//! explicit-tab active, goto and two finite watch samples with the advertised
//! 16 MiB response budget. Encrypted runs take the key from the fixture's
//! published key file and the test password from an environment variable, so
//! secrets never appear in argv.

use crate::Protection;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

pub const MAX_RESPONSE_BUDGET: &str = "16777216";

#[derive(Debug, Error)]
pub enum SmokeError {
    #[error("CLI {command} failed: {detail}")]
    CommandFailed { command: String, detail: String },
    #[error("CLI {command} produced invalid output: {detail}")]
    InvalidOutput { command: String, detail: String },
}

pub struct SmokeOptions {
    pub cli: PathBuf,
    pub socket: PathBuf,
    pub key_file: PathBuf,
    pub password: String,
    pub mode: Protection,
    /// Base environment (the fixture's); the password is injected here.
    pub env: BTreeMap<String, String>,
    pub per_query: Duration,
}

/// Run the journeys and return the case record the matrix embeds.
pub async fn run(options: &SmokeOptions) -> Value {
    match journeys(options).await {
        Ok(detail) => json!({
            "case": "cli-standalone-list-focus-watch",
            "status": "pass",
            "profile": {"auth": options.mode.as_str(), "max_response": MAX_RESPONSE_BUDGET.parse::<u64>().unwrap(), "watch_count": 2},
            "detail": detail,
        }),
        Err(error) => json!({
            "case": "cli-standalone-list-focus-watch",
            "status": "fail",
            "profile": {"auth": options.mode.as_str()},
            "detail": {"error": crate::redact_password(&error.to_string(), &options.password)},
        }),
    }
}

fn common_arguments(options: &SmokeOptions) -> Vec<String> {
    let mut args = vec![
        "--to".into(),
        options.socket.display().to_string(),
        "--auth".into(),
        options.mode.as_str().into(),
        "--max-response".into(),
        MAX_RESPONSE_BUDGET.into(),
        "--json".into(),
    ];
    if options.mode == Protection::Encrypted {
        args.push("--key-file".into());
        args.push(options.key_file.display().to_string());
        args.push("--password-env".into());
        args.push("MATRIX_TEST_PASSWORD".into());
    }
    args
}

async fn query(options: &SmokeOptions, arguments: &[&str]) -> Result<String, SmokeError> {
    let command_name = |name: &str| name.to_owned();
    let mut env = options.env.clone();
    env.insert("MATRIX_TEST_PASSWORD".into(), options.password.clone());
    let mut argv = common_arguments(options);
    argv.extend(arguments.iter().map(|value| (*value).to_owned()));
    let output = crate::examples::run_bounded(&options.cli, &argv, &env, options.per_query)
        .await
        .map_err(|error| SmokeError::CommandFailed {
            command: command_name("kitty-rc"),
            detail: error.to_string(),
        })?;
    if !output.status.success() {
        return Err(SmokeError::CommandFailed {
            command: command_name(arguments.first().copied().unwrap_or("kitty-rc")),
            detail: crate::redact_password(output.stderr.trim(), &options.password),
        });
    }
    Ok(output.stdout)
}

async fn journeys(options: &SmokeOptions) -> Result<Value, SmokeError> {
    let rows_text = query(options, &["ls"]).await?;
    let rows: Vec<Value> =
        serde_json::from_str(&rows_text).map_err(|error| SmokeError::InvalidOutput {
            command: "ls".into(),
            detail: error.to_string(),
        })?;
    let row = rows.first().ok_or(SmokeError::InvalidOutput {
        command: "ls".into(),
        detail: "no private window listed".into(),
    })?;
    let tab_id = row["tab_id"].as_u64().ok_or(SmokeError::InvalidOutput {
        command: "ls".into(),
        detail: "listed row lacks tab_id".into(),
    })?;
    let window_id = row["id"].as_u64().ok_or(SmokeError::InvalidOutput {
        command: "ls".into(),
        detail: "listed row lacks id".into(),
    })?;
    let active_text = query(options, &["active", "--tab", &tab_id.to_string()]).await?;
    let active: Value =
        serde_json::from_str(&active_text).map_err(|error| SmokeError::InvalidOutput {
            command: "active".into(),
            detail: error.to_string(),
        })?;
    if active["id"].as_u64() != Some(window_id) {
        return Err(SmokeError::InvalidOutput {
            command: "active".into(),
            detail: "explicit tab active selected a different window".into(),
        });
    }
    query(options, &["goto", &window_id.to_string()]).await?;
    let watch_text = query(options, &["watch", "--interval-ms", "20", "--count", "2"]).await?;
    let samples: Vec<Value> = watch_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|error| SmokeError::InvalidOutput {
            command: "watch".into(),
            detail: error.to_string(),
        })?;
    if samples.len() != 2
        || samples.iter().any(|sample| {
            sample.get("active").is_none()
                && sample.get("active_status").and_then(|s| s.as_str()) != Some("no-focused")
        })
    {
        return Err(SmokeError::InvalidOutput {
            command: "watch".into(),
            detail: "did not emit two valid focus/absence samples".into(),
        });
    }
    Ok(json!({
        "listed": rows.len(),
        "explicit_tab_active": true,
        "goto_ack": true,
        "watch_samples": samples.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_queries_use_key_file_and_password_env_only() {
        let options = SmokeOptions {
            cli: PathBuf::from("/bin/true"),
            socket: PathBuf::from("/f/rc.sock"),
            key_file: PathBuf::from("/f/key"),
            password: "secret-pw".into(),
            mode: Protection::Encrypted,
            env: BTreeMap::new(),
            per_query: Duration::from_secs(1),
        };
        let args = common_arguments(&options);
        assert!(args.contains(&"--password-env".to_owned()));
        assert!(args.contains(&"MATRIX_TEST_PASSWORD".to_owned()));
        assert!(args.contains(&"--key-file".into()));
        // The password itself must never appear in argv.
        assert!(!args.iter().any(|argument| argument.contains("secret-pw")));
    }

    #[test]
    fn plain_queries_pass_no_secret_arguments() {
        let options = SmokeOptions {
            cli: PathBuf::from("/bin/true"),
            socket: PathBuf::from("/f/rc.sock"),
            key_file: PathBuf::from("/f/key"),
            password: "secret-pw".into(),
            mode: Protection::Plain,
            env: BTreeMap::new(),
            per_query: Duration::from_secs(1),
        };
        let args = common_arguments(&options);
        assert!(args.contains(&"--auth".to_owned()) && args.contains(&"plain".to_owned()));
        assert!(!args.iter().any(|argument| argument.starts_with("--key")));
        assert!(!args.iter().any(|argument| argument.contains("secret-pw")));
    }
}
