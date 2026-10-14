//! Selected sources acquire material, a pure core validates its route binding
//! and key, then explicit policy composes protection. Errors never downgrade to
//! plaintext or fall through to a different source; only missing material may.
use kitty_rc_crypto::{CryptoError, ServerPublicKey};
use kitty_rc_discovery::{CapturedEnvironment, DiscoveryError, EnvironmentSource};
use kitty_rc_endpoint::{AccessBinding, Endpoint, Protection};
use std::{ffi::OsString, future::Future, path::PathBuf, pin::Pin};

pub mod sources;
pub use sources::{KeyFile, PasswordFile, StoreKey};

pub type KeyFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<KeyMaterial>, CredentialError>> + Send + 'a>>;
pub type PasswordFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<PasswordMaterial>, CredentialError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Explicit {
        label: String,
    },
    Environment {
        variable: String,
    },
    File {
        path: PathBuf,
    },
    Store {
        path: PathBuf,
        pid: u32,
        window_id: Option<u32>,
        timestamp: u64,
    },
}
impl Origin {
    /// Debug-style origin text is diagnostic provenance, not server identity.
    pub fn label(&self) -> String {
        format!("{self:?}")
    }
}
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    pub binding: AccessBinding,
    pub published: String,
    pub origin: Origin,
}
#[derive(Clone)]
pub struct PasswordMaterial {
    pub value: String,
    pub origin: Origin,
}
impl std::fmt::Debug for PasswordMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordMaterial")
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("required {kind} unavailable from selected sources")]
    Missing { kind: &'static str },
    #[error("public key from {origin:?} is scoped to {actual:?}, not {selected:?}")]
    BindingMismatch {
        origin: Origin,
        selected: Box<AccessBinding>,
        actual: Box<AccessBinding>,
    },
    #[error("KITTY_PUBLIC_KEY has no selected KITTY_LISTEN_ON binding")]
    UnboundEnvironmentKey,
    #[error("selected material from {origin:?} is not Unicode")]
    Encoding { origin: Origin },
    #[error("invalid public key from {origin:?}: {source}")]
    Key {
        origin: Origin,
        #[source]
        source: CryptoError,
    },
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
    #[error(transparent)]
    Store(#[from] kitty_rc_key_store::storage::StoreError),
    #[error("malformed key store {path:?}: {lines:?}")]
    MalformedStore {
        path: PathBuf,
        lines: Vec<(usize, kitty_rc_key_store::RecordError)>,
    },
    #[error("read credential file {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("credential file {path:?} exceeds {max_bytes} bytes")]
    Limit { path: PathBuf, max_bytes: usize },
    #[error("credential file must contain a single line: {path:?}")]
    Multiline { path: PathBuf },
    #[error("credential file is not a regular file: {path:?}")]
    NotRegular { path: PathBuf },
    #[error("credential blocking adapter stopped: {0}")]
    Adapter(#[from] tokio::task::JoinError),
}

pub trait KeySource: Send + Sync {
    fn acquire<'a>(&'a self, selected: &'a AccessBinding) -> KeyFuture<'a>;
}
pub trait PasswordSource: Send + Sync {
    fn acquire(&self) -> PasswordFuture<'_>;
}
impl KeySource for KeyMaterial {
    fn acquire<'a>(&'a self, selected: &'a AccessBinding) -> KeyFuture<'a> {
        Box::pin(async {
            check_binding(selected, &self.binding, &self.origin)?;
            Ok(Some(self.clone()))
        })
    }
}
impl PasswordSource for PasswordMaterial {
    fn acquire(&self) -> PasswordFuture<'_> {
        Box::pin(async { Ok(Some(self.clone())) })
    }
}

/// Capture related environment variables together. A key is usable only for
/// the exact published route, never for every configured kitty server.
#[derive(Debug, Clone)]
pub struct CapturedKittyKey {
    pub environment: CapturedEnvironment,
    pub public_key: Option<OsString>,
}
impl CapturedKittyKey {
    pub fn capture(source: &impl EnvironmentSource) -> Self {
        Self {
            environment: CapturedEnvironment::capture(source),
            public_key: source.get("KITTY_PUBLIC_KEY"),
        }
    }
    pub fn material(&self) -> Result<Option<KeyMaterial>, CredentialError> {
        let Some(raw) = &self.public_key else {
            return Ok(None);
        };
        let candidate = self
            .environment
            .candidates()?
            .into_iter()
            .next()
            .ok_or(CredentialError::UnboundEnvironmentKey)?;
        let origin = Origin::Environment {
            variable: "KITTY_PUBLIC_KEY".into(),
        };
        let published = raw
            .to_str()
            .ok_or_else(|| CredentialError::Encoding {
                origin: origin.clone(),
            })?
            .to_owned();
        Ok(Some(KeyMaterial {
            binding: candidate.binding,
            published,
            origin,
        }))
    }
}
impl KeySource for CapturedKittyKey {
    fn acquire<'a>(&'a self, selected: &'a AccessBinding) -> KeyFuture<'a> {
        Box::pin(async {
            let material = self.material()?;
            if let Some(material) = &material {
                check_binding(selected, &material.binding, &material.origin)?;
            }
            Ok(material)
        })
    }
}

#[derive(Clone)]
pub struct CapturedPassword {
    value: Option<OsString>,
    pub variable: String,
}
impl CapturedPassword {
    /// Name is caller-selected; there is no implicit password variable.
    pub fn capture(source: &impl EnvironmentSource, variable: &str) -> Self {
        Self {
            value: source.get(variable),
            variable: variable.into(),
        }
    }
}
impl PasswordSource for CapturedPassword {
    fn acquire(&self) -> PasswordFuture<'_> {
        Box::pin(async {
            let origin = Origin::Environment {
                variable: self.variable.clone(),
            };
            self.value
                .as_ref()
                .map(|value| {
                    value
                        .to_str()
                        .map(|s| PasswordMaterial {
                            value: s.into(),
                            origin: origin.clone(),
                        })
                        .ok_or(CredentialError::Encoding { origin })
                })
                .transpose()
        })
    }
}

/// Pure validation. Absence of a selected PID hint does not manufacture one;
/// an explicit source can declare its route association, recorded in provenance.
pub fn validate_key(
    selected: &AccessBinding,
    material: &KeyMaterial,
) -> Result<ServerPublicKey, CredentialError> {
    check_binding(selected, &material.binding, &material.origin)?;
    ServerPublicKey::parse(&material.published).map_err(|source| CredentialError::Key {
        origin: material.origin.clone(),
        source,
    })
}

fn check_binding(
    selected: &AccessBinding,
    actual: &AccessBinding,
    origin: &Origin,
) -> Result<(), CredentialError> {
    if selected.address != actual.address
        || (selected.pid_hint.is_some()
            && actual.pid_hint.is_some()
            && selected.pid_hint != actual.pid_hint)
    {
        return Err(CredentialError::BindingMismatch {
            origin: origin.clone(),
            selected: Box::new(selected.clone()),
            actual: Box::new(actual.clone()),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingPolicy {
    Stop,
    TryNext,
}
pub enum Policy<'a> {
    Plain,
    Encrypted {
        keys: &'a [&'a dyn KeySource],
        passwords: &'a [&'a dyn PasswordSource],
        missing: MissingPolicy,
    },
}
#[derive(Debug, Clone)]
pub struct Provenance {
    pub key: Option<Origin>,
    pub password: Option<Origin>,
}
#[derive(Debug, Clone)]
pub struct Resolved {
    pub binding: AccessBinding,
    pub protection: Protection,
    pub provenance: Provenance,
}
impl Resolved {
    pub fn into_endpoint(self) -> Endpoint {
        Endpoint {
            address: self.binding.address,
            protection: self.protection,
        }
    }
}
pub async fn resolve(
    binding: &AccessBinding,
    policy: Policy<'_>,
) -> Result<Resolved, CredentialError> {
    let Policy::Encrypted {
        keys,
        passwords,
        missing,
    } = policy
    else {
        return Ok(Resolved {
            binding: binding.clone(),
            protection: Protection::Plain,
            provenance: Provenance {
                key: None,
                password: None,
            },
        });
    };
    let mut selected_key = None;
    for source in keys {
        match source.acquire(binding).await? {
            Some(material) => {
                let key = validate_key(binding, &material)?;
                selected_key = Some((key, material.origin));
                break;
            }
            None if missing == MissingPolicy::Stop => break,
            None => {}
        }
    }
    let (key, key_origin) = selected_key.ok_or(CredentialError::Missing {
        kind: "server public key",
    })?;
    let mut password = None;
    for source in passwords {
        match source.acquire().await? {
            Some(material) => {
                password = Some(material);
                break;
            }
            None if missing == MissingPolicy::Stop => break,
            None => {}
        }
    }
    let password = password.ok_or(CredentialError::Missing { kind: "password" })?;
    Ok(Resolved {
        binding: binding.clone(),
        protection: Protection::Encrypted {
            key,
            password: password.value,
            key_origin: key_origin.label(),
        },
        provenance: Provenance {
            key: Some(key_origin),
            password: Some(password.origin),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kitty_rc_endpoint::Address;
    use sources::one_line;
    use std::path::Path;
    fn binding(path: &str) -> AccessBinding {
        AccessBinding {
            address: Address::Unix(path.into()),
            pid_hint: Some(7),
        }
    }
    fn key(binding: AccessBinding) -> KeyMaterial {
        let mut bytes = [0; 32];
        bytes[0] = 9;
        KeyMaterial {
            binding,
            published: format!("1:{}", base85::encode(&bytes)),
            origin: Origin::Explicit {
                label: "test".into(),
            },
        }
    }
    fn password() -> PasswordMaterial {
        PasswordMaterial {
            value: "secret".into(),
            origin: Origin::Explicit {
                label: "test".into(),
            },
        }
    }
    #[tokio::test]
    async fn sources_are_ordered_missing_is_distinct_and_errors_do_not_fall_through() {
        let selected = binding("/one");
        let good = key(selected.clone());
        let absent = CapturedKittyKey {
            environment: CapturedEnvironment::default(),
            public_key: None,
        };
        let pw = password();
        let result = resolve(
            &selected,
            Policy::Encrypted {
                keys: &[&absent, &good],
                passwords: &[&pw],
                missing: MissingPolicy::TryNext,
            },
        )
        .await
        .unwrap();
        assert!(matches!(result.protection, Protection::Encrypted { .. }));
        assert!(!format!("{result:?}").contains("secret"));
        assert!(matches!(
            resolve(
                &selected,
                Policy::Encrypted {
                    keys: &[&absent, &good],
                    passwords: &[&pw],
                    missing: MissingPolicy::Stop
                }
            )
            .await,
            Err(CredentialError::Missing {
                kind: "server public key"
            })
        ));
        let bad = KeyMaterial {
            published: "malformed".into(),
            ..good.clone()
        };
        assert!(matches!(
            resolve(
                &selected,
                Policy::Encrypted {
                    keys: &[&bad, &good],
                    passwords: &[&pw],
                    missing: MissingPolicy::TryNext
                }
            )
            .await,
            Err(CredentialError::Key { .. })
        ));
        let wrong = key(binding("/two"));
        assert!(matches!(
            resolve(
                &selected,
                Policy::Encrypted {
                    keys: &[&wrong, &good],
                    passwords: &[&pw],
                    missing: MissingPolicy::TryNext
                }
            )
            .await,
            Err(CredentialError::BindingMismatch { .. })
        ));
    }
    #[test]
    fn captured_public_key_is_not_global_or_unbound() {
        let selected = binding("/chosen");
        let good = key(selected.clone());
        let captured = CapturedKittyKey {
            environment: CapturedEnvironment {
                listen_on: Some("unix:/chosen".into()),
                pid: Some("7".into()),
            },
            public_key: Some(good.published.into()),
        };
        let material = captured.material().unwrap().unwrap();
        validate_key(&selected, &material).unwrap();
        assert!(matches!(
            validate_key(&binding("/other"), &material),
            Err(CredentialError::BindingMismatch { .. })
        ));
        let unbound = CapturedKittyKey {
            environment: CapturedEnvironment::default(),
            ..captured
        };
        assert!(matches!(
            unbound.material(),
            Err(CredentialError::UnboundEnvironmentKey)
        ));
    }

    #[tokio::test]
    async fn explicit_store_reads_legacy_without_mutation_or_helper_and_reports_all_errors() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = PathBuf::from(format!(
            "/tmp/opencode/kitty-credential-store-{}-{}.tsv",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let selected = binding("/selected");
        let good = key(selected.clone());
        let fixture = format!(
            "7\t4\t{}\n8\t\t{}\t99\n7\t9\t{}\t20\n",
            good.published, good.published, good.published
        );
        std::fs::write(&path, &fixture).unwrap();
        let source = StoreKey {
            binding: selected.clone(),
            path: path.clone(),
            max_bytes: 4096,
        };
        let material = source.acquire(&selected).await.unwrap().unwrap();
        assert_eq!(material.published, good.published);
        assert_eq!(
            material.origin,
            Origin::Store {
                path: path.clone(),
                pid: 7,
                window_id: Some(9),
                timestamp: 20
            }
        );
        validate_key(&selected, &material).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), fixture);
        assert!(matches!(
            source.acquire(&binding("/other")).await,
            Err(CredentialError::BindingMismatch { .. })
        ));
        std::fs::write(
            &path,
            format!("{fixture}broken\n7\t\t{}\tbad\n", good.published),
        )
        .unwrap();
        let error = source.acquire(&selected).await.unwrap_err();
        assert!(
            matches!(error, CredentialError::MalformedStore { lines, .. } if lines.iter().map(|(line,_)| *line).collect::<Vec<_>>() == vec![4,5])
        );
        std::fs::remove_file(&path).unwrap();
        assert!(source.acquire(&selected).await.unwrap().is_none());
        let source = StoreKey {
            path: "/tmp/opencode".into(),
            ..source
        };
        assert!(matches!(
            source.acquire(&selected).await,
            Err(CredentialError::Store(
                kitty_rc_key_store::storage::StoreError::NotRegular { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn scoped_file_is_checked_before_io_and_plain_policy_never_acquires() {
        struct Never;
        impl KeySource for Never {
            fn acquire<'a>(&'a self, _: &'a AccessBinding) -> KeyFuture<'a> {
                panic!("unselected source")
            }
        }
        impl PasswordSource for Never {
            fn acquire(&self) -> PasswordFuture<'_> {
                panic!("unselected source")
            }
        }
        let selected = binding("/chosen");
        assert!(matches!(
            resolve(&selected, Policy::Plain).await.unwrap().protection,
            Protection::Plain
        ));
        let wrong = KeyFile {
            binding: binding("/other"),
            path: "/not/a/readable/key".into(),
            max_bytes: 64,
        };
        assert!(matches!(
            wrong.acquire(&selected).await,
            Err(CredentialError::BindingMismatch { .. })
        ));
        let good = key(selected.clone());
        let pw = password();
        resolve(
            &selected,
            Policy::Encrypted {
                keys: &[&good, &Never],
                passwords: &[&pw, &Never],
                missing: MissingPolicy::TryNext,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            one_line(" secret \r\n".into(), Path::new("chosen")).unwrap(),
            " secret "
        );
        assert!(matches!(
            one_line("first\nsecond\n".into(), Path::new("chosen")),
            Err(CredentialError::Multiline { .. })
        ));
        let changed_pid = AccessBinding {
            pid_hint: Some(8),
            ..selected
        };
        assert!(matches!(
            validate_key(&changed_pid, &good),
            Err(CredentialError::BindingMismatch { .. })
        ));
    }
}
