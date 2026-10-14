use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{ffi::OsString, path::PathBuf};

#[derive(Clone, Copy, Debug, ValueEnum, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Auth {
    Plain,
    Encrypted,
}

#[derive(Parser)]
#[command(
    name = "kitty-rc",
    version,
    about = "Typed kitty remote control with explicit access and sampled observations"
)]
pub struct Cli {
    /// Explicit native Unix path or unix:/path / numeric tcp:host:port; bypass discovery
    #[arg(long, alias = "socket", global = true)]
    pub to: Option<OsString>,
    /// Read this declared JSON configuration (no implicit config search)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    /// Plain is the default; encryption never falls back to plaintext
    #[arg(long, value_enum, global = true)]
    pub auth: Option<Auth>,
    /// Single-line published server key, explicitly associated with selected endpoint
    #[arg(long, global = true, conflicts_with = "key_store")]
    pub key_file: Option<PathBuf>,
    /// Shared legacy TSV key store, explicitly associated with endpoint and --pid
    #[arg(long, global = true, conflicts_with = "key_file")]
    pub key_store: Option<PathBuf>,
    /// Declared kitty PID hint (required for store lookup; not verified identity)
    #[arg(long, global = true)]
    pub pid: Option<u32>,
    /// Single-line password file; avoids exposing passwords in argv
    #[arg(long, global = true, conflicts_with = "password_env")]
    pub password_file: Option<PathBuf>,
    /// Selected password variable; encrypted default is KITTY_RC_PASSWORD
    #[arg(long, global = true, conflicts_with = "password_file")]
    pub password_env: Option<String>,
    /// Absolute budget per acquisition/command in milliseconds, including queue time
    #[arg(long, default_value = "5000", value_parser = clap::value_parser!(u64).range(1..=86_400_000), global = true)]
    pub timeout_ms: u64,
    /// Maximum native response bytes; 256 B–16 MiB, with bounded registry retention
    #[arg(long, default_value = "1048576", value_parser = clap::value_parser!(u64).range(256..=16_777_216), global = true)]
    pub max_response: u64,
    /// Write machine-readable JSON (watch uses one JSON object per line)
    #[arg(long, global = true)]
    pub json: bool,
    /// Print selected endpoint and credential source provenance to stderr (never secrets)
    #[arg(long, global = true)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Clone, Default)]
pub struct Scope {
    /// Kitty window match expression; resulting absence applies only to this scope
    #[arg(long = "match")]
    pub match_window: Option<String>,
    /// Kitty tab match expression
    #[arg(long)]
    pub match_tab: Option<String>,
}
#[derive(Args, Clone, Default)]
pub struct Active {
    #[command(flatten)]
    pub scope: Scope,
    /// Select the active window in this tab, instead of global focused terminal
    #[arg(long, conflicts_with = "os_window")]
    pub tab: Option<u64>,
    /// Select active tab's active window within this OS window
    #[arg(long)]
    pub os_window: Option<u64>,
}
#[derive(Subcommand)]
pub enum Command {
    /// List windows from a finite typed ls acquisition
    #[command(aliases = ["ls", "list"])]
    ListWindows {
        #[command(flatten)]
        scope: Scope,
    },
    /// Focused terminal by default; no winner guessed from missing focus fields
    #[command(aliases = ["active", "a"])]
    ActiveWindow {
        #[command(flatten)]
        target: Active,
    },
    /// Focus an explicit kitty window ID (not desktop/compositor focus)
    #[command(alias = "g")]
    Goto { window_id: u64 },
    /// Sample finite ls replies; caller-owned Ctrl-C, not a remote event subscription
    #[command(alias = "w")]
    Watch {
        #[command(flatten)]
        target: Active,
        /// Fixed-delay polling cadence; missed polls are skipped, never replayed
        #[arg(long, default_value = "1000", value_parser = clap::value_parser!(u64).range(1..=86_400_000))]
        interval_ms: u64,
        /// Stop after this many samples; omitted means until Ctrl-C
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        count: Option<u64>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases_globals_and_bounds() {
        for alias in ["list-windows", "ls", "list", "active-window", "active", "a"] {
            let cli =
                Cli::try_parse_from(["kitty-rc", alias, "--to", "/chosen", "--json"]).unwrap();
            assert_eq!(cli.to, Some("/chosen".into()));
            assert!(cli.json);
        }
        for alias in ["goto", "g"] {
            assert!(Cli::try_parse_from(["kitty-rc", alias, "17"]).is_ok());
        }
        for alias in ["watch", "w"] {
            assert!(Cli::try_parse_from(["kitty-rc", alias, "--count", "2"]).is_ok());
        }
        for args in [
            vec!["kitty-rc", "w", "--interval-ms", "0"],
            vec!["kitty-rc", "ls", "--timeout-ms", "0"],
            vec!["kitty-rc", "ls", "--max-response", "1"],
            vec!["kitty-rc", "a", "--tab", "1", "--os-window", "2"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }
}
