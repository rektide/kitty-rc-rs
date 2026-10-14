//! Opt-in acquisition of pinned upstream kitty release binaries into the
//! ignored project scratch. Downloads are verified against the recorded
//! release digests, extraction rejects path escapes, and everything already
//! present with a matching digest is reused without network access.

use crate::release::{self, Release};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("fetching {url} failed: {detail}")]
    Fetch { url: String, detail: String },
    #[error("release digest mismatch for {path}: {actual}")]
    DigestMismatch { path: PathBuf, actual: String },
    #[error("archive {path} io failed")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("extracted executable reported unexpected version: {reported}")]
    VersionMismatch { reported: String },
    #[error("archive contains unsafe entry {entry}")]
    UnsafeEntry { entry: String },
    #[error("writing provenance failed")]
    Provenance {
        #[source]
        source: std::io::Error,
    },
}

/// Acquire (or verify already-acquired) one pinned release. Returns the
/// provenance record; idempotent when the archive digest and extracted
/// executable already match.
pub fn acquire(root: &Path, release_info: &Release) -> Result<serde_json::Value, DownloadError> {
    let scratch = release::scratch(root);
    let directory = scratch.join(release_info.version);
    std::fs::create_dir_all(&directory).map_err(|source| DownloadError::Io {
        path: directory.clone(),
        source,
    })?;
    let archive = directory.join(format!("kitty-{}-x86_64.txz", release_info.version));
    if !archive_with_digest(&archive, release_info.sha256)? {
        let pending = archive.with_extension("part");
        let url = release::archive_url(release_info.version);
        fetch(&url, &pending)?;
        let actual = release::sha256_file(&pending).map_err(|source| DownloadError::Io {
            path: pending.clone(),
            source,
        })?;
        if actual != release_info.sha256 {
            std::fs::remove_file(&pending).ok();
            return Err(DownloadError::DigestMismatch {
                path: pending,
                actual,
            });
        }
        std::fs::rename(&pending, &archive).map_err(|source| DownloadError::Io {
            path: archive.clone(),
            source,
        })?;
    }

    let app = directory.join("app");
    let executable = app.join("bin/kitty");
    if !executable.exists() {
        extract_txz(&archive, &app)?;
    }
    let executable = executable
        .canonicalize()
        .map_err(|source| DownloadError::Io {
            path: app.join("bin/kitty"),
            source,
        })?;
    let output = std::process::Command::new(&executable)
        .arg("--version")
        .output()
        .map_err(|source| DownloadError::Io {
            path: executable.clone(),
            source,
        })?;
    let reported = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success()
        || !reported.starts_with(&format!("kitty {} ", release_info.version))
    {
        return Err(DownloadError::VersionMismatch { reported });
    }

    let provenance = serde_json::json!({
        "version": release_info.version,
        "reported_version": reported,
        "source_revision": release_info.source_revision,
        "release_api": format!("https://api.github.com/repos/kovidgoyal/kitty/releases/tags/v{}", release_info.version),
        "download_url": release::archive_url(release_info.version),
        "archive_sha256": release_info.sha256,
        "executable": executable.display().to_string(),
        "executable_sha256": release::sha256_file(&executable).map_err(|source| DownloadError::Io { path: executable.clone(), source })?,
        "signature_verified": false,
    });
    std::fs::write(
        directory.join("provenance.json"),
        serde_json::to_string_pretty(&provenance).unwrap() + "\n",
    )
    .map_err(|source| DownloadError::Provenance { source })?;
    Ok(provenance)
}

fn archive_with_digest(archive: &Path, expected: &str) -> Result<bool, DownloadError> {
    if !archive.exists() {
        return Ok(false);
    }
    match release::sha256_file(archive) {
        Ok(actual) if actual == expected => Ok(true),
        Ok(actual) => Err(DownloadError::DigestMismatch {
            path: archive.to_owned(),
            actual,
        }),
        Err(source) => Err(DownloadError::Io {
            path: archive.to_owned(),
            source,
        }),
    }
}

fn fetch(url: &str, destination: &Path) -> Result<(), DownloadError> {
    let response = ureq::get(url)
        .set("User-Agent", "kitty-rc-verification")
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|error| DownloadError::Fetch {
            url: url.into(),
            detail: error.to_string(),
        })?;
    let mut reader = response.into_reader();
    let mut file = File::create(destination).map_err(|source| DownloadError::Io {
        path: destination.to_owned(),
        source,
    })?;
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| DownloadError::Fetch {
                url: url.into(),
                detail: error.to_string(),
            })?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|source| DownloadError::Io {
                path: destination.to_owned(),
                source,
            })?;
    }
    file.flush().map_err(|source| DownloadError::Io {
        path: destination.to_owned(),
        source,
    })?;
    Ok(())
}

/// Extract a `.txz` bundle with the safety of Python's `filter="data"`:
/// absolute paths, `..` components, devices and links escaping the
/// destination are rejected before anything is written.
pub fn extract_txz(archive: &Path, destination: &Path) -> Result<(), DownloadError> {
    std::fs::create_dir_all(destination).map_err(|source| DownloadError::Io {
        path: destination.to_owned(),
        source,
    })?;
    let file = File::open(archive).map_err(|source| DownloadError::Io {
        path: archive.to_owned(),
        source,
    })?;
    let decompressed = xz2::read::XzDecoder::new(file);
    let mut bundle = tar::Archive::new(decompressed);
    for entry in bundle.entries().map_err(|source| DownloadError::Io {
        path: archive.to_owned(),
        source,
    })? {
        let mut entry = entry.map_err(|source| DownloadError::Io {
            path: archive.to_owned(),
            source,
        })?;
        let path = entry
            .path()
            .map_err(|source| DownloadError::Io {
                path: archive.to_owned(),
                source,
            })?
            .into_owned();
        let components: Vec<_> = path.components().collect();
        if components.iter().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return Err(DownloadError::UnsafeEntry {
                entry: path.display().to_string(),
            });
        }
        match entry.header().entry_type() {
            tar::EntryType::Regular | tar::EntryType::Directory => {}
            tar::EntryType::Symlink | tar::EntryType::Link => {
                // Lexical `starts_with` does not normalize `..`, so reject
                // any parent component outright (stricter than needed and
                // unambiguous, matching the intent of Python's data filter).
                let target = entry
                    .link_name()
                    .map_err(|source| DownloadError::Io {
                        path: archive.to_owned(),
                        source,
                    })?
                    .unwrap_or_default();
                if target.is_absolute()
                    || target
                        .components()
                        .any(|component| matches!(component, std::path::Component::ParentDir))
                {
                    return Err(DownloadError::UnsafeEntry {
                        entry: format!("{} -> {}", path.display(), target.display()),
                    });
                }
            }
            other => {
                return Err(DownloadError::UnsafeEntry {
                    entry: format!("{} (type {other:?})", path.display()),
                });
            }
        }
        entry
            .unpack_in(destination)
            .map_err(|source| DownloadError::Io {
                path: destination.join(path),
                source,
            })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny valid .tar (uncompressed) built in-memory for safety checks.
    fn build_tar(entries: &[(&str, &str, tar::EntryType, Option<&str>)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, contents, kind, link) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            let data: &[u8] = contents.as_bytes();
            match link {
                Some(target) => {
                    header.set_entry_type(*kind);
                    header.set_size(0);
                    builder.append_link(&mut header, name, target).unwrap();
                }
                None => {
                    header.set_entry_type(*kind);
                    builder.append_data(&mut header, name, data).unwrap();
                }
            }
        }
        builder.into_inner().unwrap()
    }

    fn compress(uncompressed: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = xz2::write::XzEncoder::new(&mut compressed, 6);
        encoder.write_all(uncompressed).unwrap();
        encoder.finish().unwrap();
        compressed
    }

    /// Rewrite the first tar member's name to a parent escape and fix the
    /// checksum: tar-rs refuses to *create* such archives, which is exactly
    /// why the extractor must be tested against one built by hand.
    fn patch_first_member_name(uncompressed: &mut [u8], name: &[u8]) {
        for byte in &mut uncompressed[0..100] {
            *byte = 0;
        }
        uncompressed[..name.len()].copy_from_slice(name);
        for byte in &mut uncompressed[148..156] {
            *byte = b' ';
        }
        let checksum: u32 = uncompressed[..512].iter().map(|byte| *byte as u32).sum();
        uncompressed[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
    }

    #[test]
    fn rejects_parent_directory_escape() {
        let mut uncompressed = build_tar(&[("escape.txt", "x", tar::EntryType::Regular, None)]);
        patch_first_member_name(&mut uncompressed, b"../escape.txt");
        let archive_path = std::env::temp_dir().join("kitty-rc-verify-escape.txz");
        std::fs::write(&archive_path, compress(&uncompressed)).unwrap();
        let destination = std::env::temp_dir().join("kitty-rc-verify-escape-dir");
        let _ = std::fs::remove_dir_all(&destination);
        let error = extract_txz(&archive_path, &destination).unwrap_err();
        assert!(
            matches!(error, DownloadError::UnsafeEntry { .. }),
            "saw {error:?}"
        );
        assert!(!destination.join("../escape.txt").exists());
        std::fs::remove_file(&archive_path).ok();
    }

    #[test]
    fn extracts_regular_files() {
        let archive_path = std::env::temp_dir().join("kitty-rc-verify-ok.txz");
        std::fs::write(
            &archive_path,
            compress(&build_tar(&[(
                "bin/kitty",
                "#!/bin/sh\necho kitty 0.0.0\n",
                tar::EntryType::Regular,
                None,
            )])),
        )
        .unwrap();
        let destination = std::env::temp_dir().join("kitty-rc-verify-ok-dir");
        let _ = std::fs::remove_dir_all(&destination);
        extract_txz(&archive_path, &destination).unwrap();
        assert!(destination.join("bin/kitty").is_file());
        std::fs::remove_file(&archive_path).ok();
        std::fs::remove_dir_all(&destination).ok();
    }
}
