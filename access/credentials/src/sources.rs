//! Explicit, bounded I/O adapters; policy and key/binding validation stay in
//! the parent pure core. No store is located or source selected implicitly.
use crate::{
    CredentialError, KeyFuture, KeyMaterial, KeySource, Origin, PasswordFuture, PasswordMaterial,
    PasswordSource, check_binding,
};
use kitty_rc_endpoint::AccessBinding;
use std::path::{Path, PathBuf};

/// Exact route scope is explicit, even though the legacy store is keyed by a
/// PID hint. PID reuse/liveness and endpoint ownership are not proven here.
pub struct StoreKey {
    pub binding: AccessBinding,
    pub path: PathBuf,
    pub max_bytes: usize,
}
impl KeySource for StoreKey {
    fn acquire<'a>(&'a self, selected: &'a AccessBinding) -> KeyFuture<'a> {
        Box::pin(async {
            check_binding(
                selected,
                &self.binding,
                &Origin::File {
                    path: self.path.clone(),
                },
            )?;
            let pid = self.binding.pid_hint.ok_or(CredentialError::Missing {
                kind: "store PID binding",
            })?;
            let (path, max_bytes) = (self.path.clone(), self.max_bytes);
            let contents = tokio::task::spawn_blocking(move || {
                kitty_rc_key_store::storage::read(&path, max_bytes)
            })
            .await??;
            let Some(contents) = contents else {
                return Ok(None);
            };
            if !contents.malformed.is_empty() {
                return Err(CredentialError::MalformedStore {
                    path: self.path.clone(),
                    lines: contents.malformed,
                });
            }
            Ok(
                kitty_rc_key_store::latest_for_pid(&contents.records, pid).map(|record| {
                    KeyMaterial {
                        binding: self.binding.clone(),
                        published: record.pubkey.clone(),
                        origin: Origin::Store {
                            path: self.path.clone(),
                            pid,
                            window_id: record.window_id,
                            timestamp: record.timestamp,
                        },
                    }
                }),
            )
        })
    }
}

/// One UTF-8 line containing a published kitty key, scoped before filesystem I/O.
pub struct KeyFile {
    pub binding: AccessBinding,
    pub path: PathBuf,
    pub max_bytes: usize,
}
/// One UTF-8 line with an optional terminal LF/CRLF. Other whitespace is kept;
/// multiple lines are malformed, not silently truncated to a password/key.
pub struct PasswordFile {
    pub path: PathBuf,
    pub max_bytes: usize,
}
pub(crate) fn one_line(mut text: String, path: &Path) -> Result<String, CredentialError> {
    if text.ends_with('\n') {
        text.pop();
        if text.ends_with('\r') {
            text.pop();
        }
    }
    if text.contains(['\r', '\n']) {
        return Err(CredentialError::Multiline { path: path.into() });
    }
    Ok(text)
}
async fn file_text(path: &Path, max_bytes: usize) -> Result<Option<String>, CredentialError> {
    use tokio::io::AsyncReadExt;
    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CredentialError::Io {
                path: path.into(),
                source,
            });
        }
    };
    // Refuse pipes/devices/directories instead of accidentally waiting forever
    // for an unrequested streaming source. This is not a filesystem sandbox.
    if !metadata.is_file() {
        return Err(CredentialError::NotRegular { path: path.into() });
    }
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CredentialError::Io {
                path: path.into(),
                source,
            });
        }
    };
    if !file
        .metadata()
        .await
        .map_err(|source| CredentialError::Io {
            path: path.into(),
            source,
        })?
        .is_file()
    {
        return Err(CredentialError::NotRegular { path: path.into() });
    }
    let mut bytes = vec![];
    file.take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .await
        .map_err(|source| CredentialError::Io {
            path: path.into(),
            source,
        })?;
    if bytes.len() > max_bytes {
        return Err(CredentialError::Limit {
            path: path.into(),
            max_bytes,
        });
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| CredentialError::Encoding {
            origin: Origin::File { path: path.into() },
        })
}
impl KeySource for KeyFile {
    fn acquire<'a>(&'a self, selected: &'a AccessBinding) -> KeyFuture<'a> {
        Box::pin(async {
            check_binding(
                selected,
                &self.binding,
                &Origin::File {
                    path: self.path.clone(),
                },
            )?;
            file_text(&self.path, self.max_bytes)
                .await?
                .map(|text| {
                    Ok(KeyMaterial {
                        binding: self.binding.clone(),
                        published: one_line(text, &self.path)?,
                        origin: Origin::File {
                            path: self.path.clone(),
                        },
                    })
                })
                .transpose()
        })
    }
}
impl PasswordSource for PasswordFile {
    fn acquire(&self) -> PasswordFuture<'_> {
        Box::pin(async {
            file_text(&self.path, self.max_bytes)
                .await?
                .map(|text| {
                    Ok(PasswordMaterial {
                        value: one_line(text, &self.path)?,
                        origin: Origin::File {
                            path: self.path.clone(),
                        },
                    })
                })
                .transpose()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kitty_rc_endpoint::Address;
    #[tokio::test]
    async fn selected_regular_files_are_bounded_missing_malformed_and_operational_are_distinct() {
        use std::os::unix::ffi::OsStringExt;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let mut bytes = format!(
            "/tmp/opencode/kitty-key-file-{}-{}-",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
        .into_bytes();
        bytes.push(0xff);
        let path: PathBuf = std::ffi::OsString::from_vec(bytes).into();
        let selected = AccessBinding {
            address: Address::Unix("/selected".into()),
            pid_hint: None,
        };
        let source = KeyFile {
            binding: selected.clone(),
            path: path.clone(),
            max_bytes: 128,
        };
        assert!(source.acquire(&selected).await.unwrap().is_none());
        let mut raw_key = [0; 32];
        raw_key[0] = 9;
        let published = format!("1:{}", base85::encode(&raw_key));
        std::fs::write(&path, format!("{published}\r\n")).unwrap();
        let material = source.acquire(&selected).await.unwrap().unwrap();
        assert_eq!(material.published, published);
        assert_eq!(material.origin, Origin::File { path: path.clone() });
        crate::validate_key(&selected, &material).unwrap();
        std::fs::write(&path, vec![b'x'; 129]).unwrap();
        assert!(matches!(
            source.acquire(&selected).await,
            Err(CredentialError::Limit { max_bytes: 128, .. })
        ));
        std::fs::write(&path, [0xff]).unwrap();
        assert!(matches!(
            source.acquire(&selected).await,
            Err(CredentialError::Encoding { .. })
        ));
        std::fs::write(&path, "line1\nline2\n").unwrap();
        assert!(matches!(
            source.acquire(&selected).await,
            Err(CredentialError::Multiline { .. })
        ));
        let not_directory = KeyFile {
            path: path.join("child"),
            ..source
        };
        assert!(
            matches!(not_directory.acquire(&selected).await, Err(CredentialError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotADirectory)
        );
        let directory = PasswordFile {
            path: "/tmp/opencode".into(),
            max_bytes: 128,
        };
        assert!(matches!(
            directory.acquire().await,
            Err(CredentialError::NotRegular { .. })
        ));
        let password = PasswordFile {
            path: path.clone(),
            max_bytes: 128,
        };
        std::fs::write(&path, " explicit whitespace \n").unwrap();
        assert_eq!(
            password.acquire().await.unwrap().unwrap().value,
            " explicit whitespace "
        );
        std::fs::remove_file(path).unwrap();
    }
}
