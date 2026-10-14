//! Application policy and output over independent access, registry and listing leaves.
pub mod configuration;
pub mod control;
pub mod interface;
pub mod output;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
