//! Explicit candidate acquisition followed by pure selection. Nothing here
//! connects, executes a probe, guesses PID socket names, or spawns a helper.
use kitty_rc_endpoint::{AccessBinding, Address, AddressError};
use std::{ffi::OsString, future::Future, path::PathBuf, pin::Pin};

pub type SourceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<Candidate>, DiscoveryError>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Explicit,
    Environment,
    Configured,
    Convention,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Explicit,
    Environment {
        variable: String,
    },
    Configured {
        label: String,
    },
    /// A caller-declared naming convention, not discovered server identity.
    Convention {
        description: String,
    },
}
impl Origin {
    pub fn kind(&self) -> SourceKind {
        match self {
            Self::Explicit => SourceKind::Explicit,
            Self::Environment { .. } => SourceKind::Environment,
            Self::Configured { .. } => SourceKind::Configured,
            Self::Convention { .. } => SourceKind::Convention,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub binding: AccessBinding,
    pub origin: Origin,
}
impl Candidate {
    pub fn explicit(address: Address) -> Self {
        Self {
            binding: AccessBinding {
                address,
                pid_hint: None,
            },
            origin: Origin::Explicit,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("invalid selected address from {origin:?}: {source}")]
    Address {
        origin: Origin,
        #[source]
        source: AddressError,
    },
    #[error("selected KITTY_PID is not a u32")]
    InvalidPid,
    #[error("{operation} at {path:?}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("configured path is not a Unix socket: {0:?}")]
    NotSocket(PathBuf),
    #[error("candidate limit exceeded")]
    Limit,
}

/// Injectable selected environment reads. Adapters never mutate the process
/// environment. The real adapter reads only when explicitly passed to capture.
pub trait EnvironmentSource {
    fn get(&self, name: &str) -> Option<OsString>;
}
pub struct ProcessEnvironment;
impl EnvironmentSource for ProcessEnvironment {
    fn get(&self, name: &str) -> Option<OsString> {
        std::env::var_os(name)
    }
}
impl EnvironmentSource for std::collections::BTreeMap<String, OsString> {
    fn get(&self, name: &str) -> Option<OsString> {
        self.get(name).cloned()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapturedEnvironment {
    pub listen_on: Option<OsString>,
    pub pid: Option<OsString>,
}
impl CapturedEnvironment {
    pub fn capture(source: &impl EnvironmentSource) -> Self {
        Self {
            listen_on: source.get("KITTY_LISTEN_ON"),
            pid: source.get("KITTY_PID"),
        }
    }
    /// Pure interpretation of captured inputs. `fd:`/abstract routes are
    /// explicitly unsupported, never replaced by a guessed filesystem socket.
    pub fn candidates(&self) -> Result<Vec<Candidate>, DiscoveryError> {
        let Some(raw) = &self.listen_on else {
            return Ok(vec![]);
        };
        let origin = Origin::Environment {
            variable: "KITTY_LISTEN_ON".into(),
        };
        let address = Address::parse_native(raw).map_err(|source| DiscoveryError::Address {
            origin: origin.clone(),
            source,
        })?;
        let pid_hint = self
            .pid
            .as_ref()
            .map(|raw| {
                raw.to_str()
                    .and_then(|s| s.parse().ok())
                    .ok_or(DiscoveryError::InvalidPid)
            })
            .transpose()?;
        Ok(vec![Candidate {
            binding: AccessBinding { address, pid_hint },
            origin,
        }])
    }
}

pub trait CandidateSource: Send + Sync {
    fn acquire(&self) -> SourceFuture<'_>;
}
impl CandidateSource for CapturedEnvironment {
    fn acquire(&self) -> SourceFuture<'_> {
        Box::pin(async { self.candidates() })
    }
}
impl CandidateSource for Vec<Candidate> {
    fn acquire(&self) -> SourceFuture<'_> {
        Box::pin(async { Ok(self.clone()) })
    }
}

/// Selected paths only: no globbing, directory sweep, environment expansion or
/// PID suffix inference. Missing paths are absent candidates; every other I/O
/// failure and non-socket entry is an error rather than silent fallback.
pub struct SocketPaths {
    pub paths: Vec<PathBuf>,
    pub label: String,
}
impl CandidateSource for SocketPaths {
    fn acquire(&self) -> SourceFuture<'_> {
        Box::pin(async {
            use std::os::unix::fs::FileTypeExt;
            let mut candidates = vec![];
            for path in &self.paths {
                let metadata = match tokio::fs::metadata(path).await {
                    Ok(value) => value,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(source) => {
                        return Err(DiscoveryError::Io {
                            operation: "inspect socket",
                            path: path.clone(),
                            source,
                        });
                    }
                };
                if !metadata.file_type().is_socket() {
                    return Err(DiscoveryError::NotSocket(path.clone()));
                }
                candidates.push(Candidate {
                    binding: AccessBinding {
                        address: Address::Unix(path.clone()),
                        pid_hint: None,
                    },
                    origin: Origin::Configured {
                        label: self.label.clone(),
                    },
                });
            }
            Ok(candidates)
        })
    }
}

#[derive(Debug, Clone)]
pub enum SelectionPolicy {
    /// More than one candidate is ambiguous, including aliases with different
    /// provenance. Selection never silently merges identities or hints.
    Unique,
    /// Caller explicitly accepts acquisition order as the tie-breaker.
    First,
    /// First nonempty source tier; ties within that tier remain ambiguous.
    Prefer(Vec<SourceKind>),
}
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SelectionError {
    #[error("no endpoint candidate selected by policy")]
    Missing,
    #[error("{count} endpoint candidates are ambiguous under policy")]
    Ambiguous { count: usize },
}
pub fn select<'a>(
    candidates: &'a [Candidate],
    policy: &SelectionPolicy,
) -> Result<&'a Candidate, SelectionError> {
    let unique = |items: Vec<&'a Candidate>| match items.as_slice() {
        [] => Err(SelectionError::Missing),
        [one] => Ok(*one),
        _ => Err(SelectionError::Ambiguous { count: items.len() }),
    };
    match policy {
        SelectionPolicy::Unique => unique(candidates.iter().collect()),
        SelectionPolicy::First => candidates.first().ok_or(SelectionError::Missing),
        SelectionPolicy::Prefer(order) => {
            for kind in order {
                let tier: Vec<_> = candidates
                    .iter()
                    .filter(|c| c.origin.kind() == *kind)
                    .collect();
                if !tier.is_empty() {
                    return unique(tier);
                }
            }
            Err(SelectionError::Missing)
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error(transparent)]
    Acquisition(#[from] DiscoveryError),
    #[error(transparent)]
    Selection(#[from] SelectionError),
}
pub async fn discover(
    explicit: Option<Candidate>,
    sources: &[&dyn CandidateSource],
    policy: &SelectionPolicy,
    max_candidates: usize,
) -> Result<Candidate, ResolveError> {
    // Explicit route bypasses even malformed/slow sources, not merely selection.
    if let Some(candidate) = explicit {
        return Ok(candidate);
    }
    let mut candidates = vec![];
    for source in sources {
        let acquired = source.acquire().await?;
        if acquired.len() > max_candidates.saturating_sub(candidates.len()) {
            return Err(DiscoveryError::Limit.into());
        }
        candidates.extend(acquired);
    }
    Ok(select(&candidates, policy)?.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_inputs_native_paths_and_pure_ordering() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let raw = OsString::from_vec(b"unix:/chosen/\xff.socket".to_vec());
        let env = CapturedEnvironment {
            listen_on: Some(raw),
            pid: Some("7".into()),
        };
        let mut candidates = env.candidates().unwrap();
        let Address::Unix(path) = &candidates[0].binding.address else {
            panic!()
        };
        assert_eq!(path.as_os_str().as_bytes(), b"/chosen/\xff.socket");
        candidates.push(Candidate {
            binding: AccessBinding {
                address: "tcp:127.0.0.1:9".parse().unwrap(),
                pid_hint: None,
            },
            origin: Origin::Configured {
                label: "chosen".into(),
            },
        });
        assert!(matches!(
            select(&candidates, &SelectionPolicy::Unique),
            Err(SelectionError::Ambiguous { count: 2 })
        ));
        assert_eq!(
            select(&candidates, &SelectionPolicy::First).unwrap(),
            &candidates[0]
        );
        assert_eq!(
            select(
                &candidates,
                &SelectionPolicy::Prefer(vec![SourceKind::Configured, SourceKind::Environment])
            )
            .unwrap(),
            &candidates[1]
        );
        assert_eq!(candidates[0].binding.pid_hint, Some(7));
        let aliases = vec![
            candidates[0].clone(),
            Candidate {
                binding: candidates[0].binding.clone(),
                origin: Origin::Configured {
                    label: "same route, separate evidence".into(),
                },
            },
        ];
        assert!(matches!(
            select(&aliases, &SelectionPolicy::Unique),
            Err(SelectionError::Ambiguous { count: 2 })
        ));
    }
    #[tokio::test]
    async fn explicit_bypasses_acquisition_and_malformed_is_not_missing() {
        struct Never;
        impl CandidateSource for Never {
            fn acquire(&self) -> SourceFuture<'_> {
                panic!("explicit bypass must not acquire")
            }
        }
        let chosen = Candidate::explicit("unix:/explicit".parse().unwrap());
        assert_eq!(
            discover(Some(chosen.clone()), &[&Never], &SelectionPolicy::Unique, 1)
                .await
                .unwrap(),
            chosen
        );
        let malformed = CapturedEnvironment {
            listen_on: Some("fd:9".into()),
            pid: None,
        };
        let fallback = vec![chosen];
        assert!(matches!(
            discover(None, &[&malformed, &fallback], &SelectionPolicy::First, 3).await,
            Err(ResolveError::Acquisition(DiscoveryError::Address { .. }))
        ));
        assert!(matches!(
            discover(
                None,
                &[&CapturedEnvironment::default()],
                &SelectionPolicy::Unique,
                3
            )
            .await,
            Err(ResolveError::Selection(SelectionError::Missing))
        ));
    }

    #[tokio::test]
    async fn configured_paths_acquire_only_actual_socket_and_fail_on_non_socket() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = PathBuf::from(format!(
            "/tmp/opencode/kitty-discovery-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let missing = path.with_extension("absent");
        let sources = SocketPaths {
            paths: vec![missing.clone(), path.clone()],
            label: "explicit configuration".into(),
        };
        let result = sources.acquire().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].binding.address, Address::Unix(path.clone()));
        assert_eq!(result[0].binding.pid_hint, None);
        assert!(matches!(
            discover(None, &[&sources], &SelectionPolicy::Unique, 0).await,
            Err(ResolveError::Acquisition(DiscoveryError::Limit))
        ));
        std::fs::write(&missing, "not a socket").unwrap();
        assert!(
            matches!(sources.acquire().await, Err(DiscoveryError::NotSocket(found)) if found == missing)
        );
        drop(listener);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(missing).unwrap();
    }
}
