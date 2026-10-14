//! Private verification orchestration for the kitty-rc workspace.
//!
//! This crate owns the full verification lifecycle that previously lived in
//! `scripts/*.py`: isolated headless Weston/kitty fixtures, bounded process
//! supervision, pinned upstream release acquisition, the supported-version
//! matrix, standalone CLI smoke journeys, design-probe orchestration and the
//! pinned source-contract oracle. The only retained Python surfaces are
//! deliberate boundaries:
//!
//! - [`fixtures/no_ui.py`](../fixtures/no_ui.py): an upstream **kitten**
//!   (kitty plugins are Python); Rust cannot be a kitten.
//! - [`fixtures/source_oracle.py`](../fixtures/source_oracle.py): the minimal
//!   executor for upstream constructors/parsers at a pinned revision. Only it
//!   executes upstream GPL source; every case and comparison lives in Rust.
//!
//! Invariants: never select an ambient human endpoint, key or config; private
//! headless fixtures only; owned children are torn down on every path
//! (normal, deadline, Ctrl-C, panic); credentials are fixture-local test
//! material, never human secrets.

pub mod childbin;
pub mod download;
pub mod env;
pub mod examples;
pub mod fixture;
pub mod kubeless;
pub mod matrix;
pub mod png;
pub mod release;
pub mod report;
pub mod smoke;
pub mod source;
pub mod supervision;

/// Protection profile a private fixture selects.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Protection {
    Encrypted,
    Plain,
}

impl Protection {
    pub fn as_str(self) -> &'static str {
        match self {
            Protection::Encrypted => "encrypted",
            Protection::Plain => "plain",
        }
    }
}

/// Replace the fixture test password in captured diagnostics.
pub fn redact_password(text: &str, password: &str) -> String {
    if password.is_empty() {
        text.to_owned()
    } else {
        text.replace(password, "<test-password>")
    }
}
