//! Application policy and output over independent access, registry and listing leaves.
pub mod configuration;
pub mod control;
pub mod interface;
pub mod output;
pub mod store;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct Interrupted;
impl std::fmt::Display for Interrupted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("interrupted by Ctrl-C; no mutation will be replayed")
    }
}
impl std::error::Error for Interrupted {}
