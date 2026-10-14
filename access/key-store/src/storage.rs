//! Explicit blocking filesystem boundary. Async callers must use their chosen
//! blocking executor; no environment, helper process, cleanup, or path guessing.
use crate::{PubkeyRecord, StoreContents, parse_store};
use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{operation} key store {path:?}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("key store {path:?} exceeds {max_bytes} bytes")]
    Limit { path: PathBuf, max_bytes: usize },
    #[error("key store {path:?} is not UTF-8: {source}")]
    Encoding {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error("public key record cannot contain TSV line/field separators")]
    RecordSeparators,
}

/// Missing file is distinct from an existing empty/malformed store. Every
/// malformed record remains reported by the shared parser; this never rewrites.
pub fn read(path: &Path, max_bytes: usize) -> Result<Option<StoreContents>, StoreError> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(StoreError::Io {
                operation: "open",
                path: path.into(),
                source,
            });
        }
    };
    let mut bytes = vec![];
    file.take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| StoreError::Io {
            operation: "read",
            path: path.into(),
            source,
        })?;
    if bytes.len() > max_bytes {
        return Err(StoreError::Limit {
            path: path.into(),
            max_bytes,
        });
    }
    let text = String::from_utf8(bytes).map_err(|source| StoreError::Encoding {
        path: path.into(),
        source,
    })?;
    Ok(Some(parse_store(&text)))
}

/// Append a publication; never replace previous PID records or malformed data.
/// Parent directory creation and cleanup are caller decisions. Separate writer
/// processes need caller-level coordination; this is not a transactional DB.
pub fn append(path: &Path, record: &PubkeyRecord) -> Result<(), StoreError> {
    if record.pubkey.contains(['\t', '\r', '\n']) {
        return Err(StoreError::RecordSeparators);
    }
    let action = || -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        let mut prefix = "";
        if file.metadata()?.len() != 0 {
            file.seek(SeekFrom::End(-1))?;
            let mut last = [0];
            file.read_exact(&mut last)?;
            if last[0] != b'\n' {
                prefix = "\n";
            }
        }
        file.write_all(format!("{prefix}{}\n", record.render()).as_bytes())?;
        file.flush()
    };
    action().map_err(|source| StoreError::Io {
        operation: "append",
        path: path.into(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    #[test]
    fn legacy_append_preserves_every_byte_and_reports_malformed_records() {
        let path = PathBuf::from(format!(
            "/tmp/opencode/kitty-store-{}-{}.tsv",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let old = b"1\t7\t1:old\t10\n2\t\t1:legacy\nbroken\n3\t\t1:x\tbad";
        std::fs::write(&path, old).unwrap();
        let record = PubkeyRecord {
            pid: 1,
            window_id: None,
            pubkey: "1:new".into(),
            timestamp: 20,
        };
        append(&path, &record).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(old));
        let contents = read(&path, 4096).unwrap().unwrap();
        assert_eq!(contents.records.len(), 3);
        assert_eq!(contents.records[1].timestamp, 0);
        assert_eq!(
            contents
                .malformed
                .iter()
                .map(|(line, _)| *line)
                .collect::<Vec<_>>(),
            vec![3, 4]
        );
        assert_eq!(crate::latest_for_pid(&contents.records, 1), Some(&record));
        assert!(matches!(read(&path, 2), Err(StoreError::Limit { .. })));
        assert!(matches!(
            append(
                &path,
                &PubkeyRecord {
                    pubkey: "bad\nrecord".into(),
                    ..record
                }
            ),
            Err(StoreError::RecordSeparators)
        ));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(&path).unwrap();
        assert!(read(&path, 4096).unwrap().is_none());
    }
}
