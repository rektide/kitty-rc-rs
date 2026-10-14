//! Shared explicit access arguments, not an automatic discovery policy.
use clap::Args;
use kitty_rc_credentials::{KeyFile, MissingPolicy, PasswordFile, Policy, Resolved};
use kitty_rc_discovery::{Candidate, SelectionPolicy, discover};
use kitty_rc_endpoint::Address;
use std::path::PathBuf;

pub type Error = Box<dyn std::error::Error>;

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct ProtectionArgs {
    /// Select plaintext deliberately; never an encryption-error fallback.
    #[arg(long)]
    plain: bool,
    /// Select encrypted requests using two explicit one-line files.
    #[arg(long, requires = "password_file")]
    key_file: Option<PathBuf>,
}

#[derive(Args)]
pub struct AccessArgs {
    /// Explicit unix:/path or tcp:IP:port; no ambient endpoint reads.
    #[arg(long)]
    pub endpoint: Address,
    #[command(flatten)]
    protection: ProtectionArgs,
    /// Password file is read only for the encrypted selection.
    #[arg(long, requires = "key_file", conflicts_with = "plain")]
    password_file: Option<PathBuf>,
}
impl AccessArgs {
    pub async fn resolve(&self) -> Result<Resolved, Error> {
        // Explicit candidate bypasses all source acquisition, including environment.
        let candidate = discover(
            Some(Candidate::explicit(self.endpoint.clone())),
            &[],
            &SelectionPolicy::Unique,
            1,
        )
        .await?;
        let resolved = match (&self.protection.key_file, &self.password_file) {
            (Some(key), Some(password)) => {
                let key = KeyFile {
                    binding: candidate.binding.clone(),
                    path: key.clone(),
                    max_bytes: 4096,
                };
                let password = PasswordFile {
                    path: password.clone(),
                    max_bytes: 4096,
                };
                kitty_rc_credentials::resolve(
                    &candidate.binding,
                    Policy::Encrypted {
                        keys: &[&key],
                        passwords: &[&password],
                        missing: MissingPolicy::Stop,
                    },
                )
                .await?
            }
            (None, None) if self.protection.plain => {
                kitty_rc_credentials::resolve(&candidate.binding, Policy::Plain).await?
            }
            _ => return Err("select --plain or both --key-file and --password-file".into()),
        };
        Ok(resolved)
    }
}
