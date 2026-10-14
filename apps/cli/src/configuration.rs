//! Small declared JSON configuration and selected environment precedence.
use crate::{
    Error,
    interface::{Auth, Cli},
};
use kitty_rc_credentials::{
    self as credentials, CapturedKittyKey, CapturedPassword, KeyFile, KeySource, MissingPolicy,
    PasswordFile, PasswordSource, Policy, StoreKey,
};
use kitty_rc_discovery::{
    self as discovery, Candidate, CapturedEnvironment, EnvironmentSource, Origin, SelectionPolicy,
};
use kitty_rc_endpoint::Address;
use serde::Deserialize;
use std::{ffi::OsString, path::PathBuf};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FileConfig {
    pub endpoints: Vec<String>,
    pub auth: Option<Auth>,
    pub key_file: Option<PathBuf>,
    pub key_store: Option<PathBuf>,
    pub pid: Option<u32>,
    pub password_file: Option<PathBuf>,
    pub password_env: Option<String>,
}
impl FileConfig {
    pub async fn read(path: Option<&PathBuf>) -> Result<Self, Error> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        // Declared files only; same bounded regular-file boundary as credential input.
        let file = tokio::fs::File::open(path).await?;
        if !file.metadata().await?.is_file() {
            return Err("configuration must be a regular file".into());
        }
        use tokio::io::AsyncReadExt;
        let mut bytes = Vec::new();
        file.take(65_537).read_to_end(&mut bytes).await?;
        if bytes.len() > 65_536 {
            return Err("configuration exceeds 64 KiB".into());
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
}

pub async fn endpoint(
    cli: &Cli,
    file: &FileConfig,
    env: &impl EnvironmentSource,
) -> Result<Candidate, Error> {
    use std::os::unix::ffi::OsStrExt;
    let explicit = cli
        .to
        .as_ref()
        .map(|raw| {
            if raw.is_empty() {
                return Err(kitty_rc_endpoint::AddressError::Scheme);
            }
            if raw.as_os_str().as_bytes().contains(&b':') {
                Address::parse_native(raw).map(Candidate::explicit)
            } else {
                Ok(Candidate::explicit(Address::Unix(PathBuf::from(raw))))
            }
        })
        .transpose()?;
    if explicit.is_some() {
        return Ok(discovery::discover(explicit, &[], &SelectionPolicy::Unique, 32).await?);
    }
    let captured = CapturedEnvironment::capture(env);
    // Don't interpret a lower-precedence configured route when selected env exists.
    if captured.listen_on.is_some() {
        return Ok(discovery::discover(None, &[&captured], &SelectionPolicy::Unique, 32).await?);
    }
    let configured: Vec<_> = file
        .endpoints
        .iter()
        .map(|raw| {
            Address::parse_native(&OsString::from(raw)).map(|address| Candidate {
                binding: kitty_rc_endpoint::AccessBinding {
                    address,
                    pid_hint: None,
                },
                origin: Origin::Configured {
                    label: "declared JSON endpoints".into(),
                },
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(
        discovery::discover(None, &[&configured], &SelectionPolicy::Unique, 32)
            .await
            .map_err(|error| {
                format!(
                    "{error}; select --to SOCKET or KITTY_LISTEN_ON (no guessed default socket)"
                )
            })?,
    )
}

enum KeyChoice {
    Environment,
    File(PathBuf),
    Store(PathBuf),
}
enum PasswordChoice {
    Environment(String),
    File(PathBuf),
}

fn unicode(env: &impl EnvironmentSource, name: &str) -> Result<Option<String>, Error> {
    env.get(name)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| format!("selected {name} is not Unicode").into())
        })
        .transpose()
}
fn auth(cli: &Cli, file: &FileConfig, env: &impl EnvironmentSource) -> Result<Auth, Error> {
    if let Some(auth) = cli.auth {
        return Ok(auth);
    }
    if let Some(value) = unicode(env, "KITTY_RC_AUTH")? {
        return match value.as_str() {
            "plain" => Ok(Auth::Plain),
            "encrypted" => Ok(Auth::Encrypted),
            _ => Err("KITTY_RC_AUTH must be plain or encrypted".into()),
        };
    }
    Ok(file.auth.unwrap_or(Auth::Plain))
}
fn key_choice(
    cli: &Cli,
    file: &FileConfig,
    env: &impl EnvironmentSource,
) -> Result<KeyChoice, Error> {
    if let Some(path) = &cli.key_file {
        return Ok(KeyChoice::File(path.clone()));
    }
    if let Some(path) = &cli.key_store {
        return Ok(KeyChoice::Store(path.clone()));
    }
    let env_file = env.get("KITTY_RC_KEY_FILE");
    let env_store = env.get("KITTY_RC_KEY_STORE");
    if env_file.is_some() && env_store.is_some() {
        return Err("select only one of KITTY_RC_KEY_FILE and KITTY_RC_KEY_STORE".into());
    }
    if let Some(path) = env_file {
        return Ok(KeyChoice::File(path.into()));
    }
    if let Some(path) = env_store {
        return Ok(KeyChoice::Store(path.into()));
    }
    if file.key_file.is_some() && file.key_store.is_some() {
        return Err("configuration must select key_file or key_store, not both".into());
    }
    if let Some(path) = &file.key_file {
        return Ok(KeyChoice::File(path.clone()));
    }
    if let Some(path) = &file.key_store {
        return Ok(KeyChoice::Store(path.clone()));
    }
    Ok(KeyChoice::Environment)
}
fn password_choice(
    cli: &Cli,
    file: &FileConfig,
    env: &impl EnvironmentSource,
) -> Result<PasswordChoice, Error> {
    if let Some(path) = &cli.password_file {
        return Ok(PasswordChoice::File(path.clone()));
    }
    if let Some(name) = &cli.password_env {
        return Ok(PasswordChoice::Environment(name.clone()));
    }
    let env_file = env.get("KITTY_RC_PASSWORD_FILE");
    let env_name = unicode(env, "KITTY_RC_PASSWORD_ENV")?;
    if env_file.is_some() && env_name.is_some() {
        return Err("select only one of KITTY_RC_PASSWORD_FILE and KITTY_RC_PASSWORD_ENV".into());
    }
    if let Some(path) = env_file {
        return Ok(PasswordChoice::File(path.into()));
    }
    if let Some(name) = env_name {
        return Ok(PasswordChoice::Environment(name));
    }
    if file.password_file.is_some() && file.password_env.is_some() {
        return Err("configuration must select password_file or password_env, not both".into());
    }
    if let Some(path) = &file.password_file {
        return Ok(PasswordChoice::File(path.clone()));
    }
    Ok(PasswordChoice::Environment(
        file.password_env
            .clone()
            .unwrap_or_else(|| "KITTY_RC_PASSWORD".into()),
    ))
}

pub async fn access(
    cli: &Cli,
    file: &FileConfig,
    env: &impl EnvironmentSource,
    mut candidate: Candidate,
) -> Result<(Candidate, credentials::Resolved), Error> {
    let auth = auth(cli, file, env)?;
    if auth == Auth::Plain {
        if cli.key_file.is_some()
            || cli.key_store.is_some()
            || cli.password_env.is_some()
            || cli.password_file.is_some()
        {
            return Err(
                "credential flags require --auth encrypted; plaintext is not a fallback".into(),
            );
        }
        let resolved = credentials::resolve(&candidate.binding, Policy::Plain).await?;
        return Ok((candidate, resolved));
    }
    let pid = if let Some(pid) = cli.pid {
        Some(pid)
    } else if let Some(raw) = unicode(env, "KITTY_RC_PID")? {
        Some(
            raw.parse::<u32>()
                .map_err(|_| "KITTY_RC_PID must be a u32")?,
        )
    } else {
        file.pid.or(candidate.binding.pid_hint)
    };
    if let (Some(selected), Some(declared)) = (candidate.binding.pid_hint, pid)
        && selected != declared
    {
        return Err("declared PID conflicts with selected endpoint's KITTY_PID hint".into());
    }
    candidate.binding.pid_hint = pid;
    let key: Box<dyn KeySource> = match key_choice(cli, file, env)? {
        KeyChoice::Environment => Box::new(CapturedKittyKey::capture(env)),
        KeyChoice::File(path) => Box::new(KeyFile {
            binding: candidate.binding.clone(),
            path,
            max_bytes: 4096,
        }),
        KeyChoice::Store(path) => {
            if pid.is_none() {
                return Err(
                    "key-store lookup requires --pid / KITTY_RC_PID / declared config pid".into(),
                );
            }
            Box::new(StoreKey {
                binding: candidate.binding.clone(),
                path,
                max_bytes: 16 * 1024 * 1024,
            })
        }
    };
    let password: Box<dyn PasswordSource> = match password_choice(cli, file, env)? {
        PasswordChoice::Environment(name) => {
            if name.is_empty() || name.contains('=') || name.contains('\0') {
                return Err("invalid selected password variable name".into());
            }
            Box::new(CapturedPassword::capture(env, &name))
        }
        PasswordChoice::File(path) => Box::new(PasswordFile {
            path,
            max_bytes: 65_536,
        }),
    };
    let resolved = credentials::resolve(
        &candidate.binding,
        Policy::Encrypted {
            keys: &[key.as_ref()],
            passwords: &[password.as_ref()],
            missing: MissingPolicy::Stop,
        },
    )
    .await?;
    Ok((candidate, resolved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::collections::BTreeMap;
    #[tokio::test]
    async fn explicit_env_config_precedence_and_ambiguity_are_pure() {
        let file = FileConfig {
            endpoints: vec!["unix:/configured".into(), "unix:/second".into()],
            ..Default::default()
        };
        let mut env = BTreeMap::<String, OsString>::new();
        env.insert("KITTY_LISTEN_ON".into(), "fd:9".into());
        env.insert("KITTY_PID".into(), "bad".into());
        let cli = Cli::parse_from(["kitty-rc", "--to", "/explicit", "ls"]);
        assert_eq!(
            endpoint(&cli, &file, &env).await.unwrap().binding.address,
            Address::Unix("/explicit".into())
        );
        let cli = Cli::parse_from(["kitty-rc", "ls"]);
        assert!(endpoint(&cli, &file, &env).await.is_err());
        env.insert("KITTY_LISTEN_ON".into(), "unix:/environment".into());
        env.remove("KITTY_PID");
        assert_eq!(
            endpoint(&cli, &file, &env).await.unwrap().origin.kind(),
            discovery::SourceKind::Environment
        );
        env.clear();
        assert!(
            endpoint(&cli, &file, &env)
                .await
                .unwrap_err()
                .to_string()
                .contains("ambiguous")
        );
        assert!(
            endpoint(&cli, &FileConfig::default(), &env)
                .await
                .unwrap_err()
                .to_string()
                .contains("--to")
        );
    }
    #[test]
    fn auth_and_source_precedence_do_not_read_lower_selected_environment() {
        let file = FileConfig {
            auth: Some(Auth::Encrypted),
            key_store: Some("/config".into()),
            password_env: Some("CONFIG_SECRET".into()),
            ..Default::default()
        };
        let env = BTreeMap::from([
            ("KITTY_RC_AUTH".into(), OsString::from("invalid")),
            ("KITTY_RC_KEY_FILE".into(), OsString::from("/env")),
            ("KITTY_RC_PASSWORD_FILE".into(), OsString::from("/env-pw")),
        ]);
        let cli = Cli::parse_from([
            "kitty-rc",
            "--auth",
            "encrypted",
            "--key-store",
            "/cli",
            "--password-env",
            "CLI_SECRET",
            "ls",
        ]);
        assert_eq!(auth(&cli, &file, &env).unwrap(), Auth::Encrypted);
        assert!(
            matches!(key_choice(&cli, &file, &env).unwrap(), KeyChoice::Store(path) if path == std::path::Path::new("/cli"))
        );
        assert!(
            matches!(password_choice(&cli, &file, &env).unwrap(), PasswordChoice::Environment(name) if name == "CLI_SECRET")
        );
        assert!(auth(&Cli::parse_from(["kitty-rc", "ls"]), &file, &env).is_err());
        let cli = Cli::parse_from(["kitty-rc", "ls"]);
        let env = BTreeMap::from([
            ("KITTY_RC_AUTH".into(), OsString::from("plain")),
            ("KITTY_RC_KEY_FILE".into(), OsString::from("/env")),
            (
                "KITTY_RC_PASSWORD_FILE".into(),
                OsString::from("/env-password"),
            ),
        ]);
        assert_eq!(auth(&cli, &file, &env).unwrap(), Auth::Plain);
        assert!(
            matches!(key_choice(&cli, &file, &env).unwrap(), KeyChoice::File(path) if path == std::path::Path::new("/env"))
        );
        assert!(
            matches!(password_choice(&cli, &file, &env).unwrap(), PasswordChoice::File(path) if path == std::path::Path::new("/env-password"))
        );
        let empty = BTreeMap::<String, OsString>::new();
        assert_eq!(auth(&cli, &file, &empty).unwrap(), Auth::Encrypted);
        assert_eq!(
            auth(&cli, &FileConfig::default(), &empty).unwrap(),
            Auth::Plain
        );
    }
}
