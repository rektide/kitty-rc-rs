//! Persisted kitty public-key records: the format behind the
//! `kitty-pubkey-db` CLI and any future credential-resolution composition.
//!
//! Format (unchanged from the existing store, which is user data): one record
//! per line, tab-separated — `pid  window_id  pubkey  timestamp`. `window_id`
//! may be empty; a missing or unparseable timestamp reads as `0`. The store
//! file conventionally lives at `$XDG_STATE_HOME/kitty/pubkey.tsv`.
//!
//! This crate is format-only and pure: no filesystem access, no process
//! liveness checks, no cleanup. Malformed lines are **reported, never
//! silently dropped** — erasing or filtering user data is a deliberate
//! caller action. Validation of the key string itself (the `1:<base85>`
//! X25519 format) belongs to `kitty-rc-crypto` at use time; a stored record
//! here is opaque text with its origin metadata.

use thiserror::Error;

/// Why a single store line could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecordError {
    #[error("expected 3 or 4 tab-separated fields, found {found}")]
    FieldCount { found: usize },

    #[error("pid {value:?} is not a u32")]
    InvalidPid { value: String },

    #[error("window id {value:?} is not a u32")]
    InvalidWindowId { value: String },

    #[error("timestamp {value:?} is not a u64")]
    InvalidTimestamp { value: String },
}

/// One stored public-key record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubkeyRecord {
    /// PID of the kitty instance the key was published for. A lookup hint
    /// with provenance, not proof of server identity; PIDs are reused.
    pub pid: u32,
    /// Kitty window id current when the key was published, when known.
    pub window_id: Option<u32>,
    /// The published key string (opaque here; `1:<base85>` for kitty today).
    pub pubkey: String,
    /// Unix seconds when the record was written.
    pub timestamp: u64,
}

/// A parsed store: usable records plus every malformed line, with its
/// 1-based line number, so callers can report rather than erase problems.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreContents {
    pub records: Vec<PubkeyRecord>,
    pub malformed: Vec<(usize, RecordError)>,
}

impl PubkeyRecord {
    /// Parse one store line.
    pub fn parse(line: &str) -> Result<Self, RecordError> {
        let line = line.trim_end_matches(['\n', '\r']);
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            [pid, window_id, pubkey] => Self::from_fields(pid, window_id, pubkey, None),
            [pid, window_id, pubkey, timestamp] => {
                Self::from_fields(pid, window_id, pubkey, Some(timestamp))
            }
            rest => Err(RecordError::FieldCount { found: rest.len() }),
        }
    }

    fn from_fields(
        pid: &str,
        window_id: &str,
        pubkey: &str,
        timestamp: Option<&str>,
    ) -> Result<Self, RecordError> {
        let pid = pid.parse().map_err(|_| RecordError::InvalidPid {
            value: pid.to_string(),
        })?;
        let window_id = if window_id.is_empty() {
            None
        } else {
            Some(
                window_id
                    .parse()
                    .map_err(|_| RecordError::InvalidWindowId {
                        value: window_id.to_string(),
                    })?,
            )
        };
        let timestamp = match timestamp {
            None => 0,
            Some(raw) => raw.parse().map_err(|_| RecordError::InvalidTimestamp {
                value: raw.to_string(),
            })?,
        };
        Ok(Self {
            pid,
            window_id,
            pubkey: pubkey.to_string(),
            timestamp,
        })
    }

    /// Render the record in the persisted TSV format. `window_id` renders as
    /// an empty field when absent, matching existing files.
    pub fn render(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.pid,
            self.window_id.map(|id| id.to_string()).unwrap_or_default(),
            self.pubkey,
            self.timestamp
        )
    }
}

/// Parse an entire store file's text. Empty lines are skipped; malformed
/// lines are collected with 1-based line numbers and do not abort parsing.
pub fn parse_store(text: &str) -> StoreContents {
    let mut contents = StoreContents::default();
    for (index, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        match PubkeyRecord::parse(line) {
            Ok(record) => contents.records.push(record),
            Err(error) => contents.malformed.push((index + 1, error)),
        }
    }
    contents
}

/// The newest record for `pid` (max timestamp), or `None` when no record
/// exists for it.
pub fn latest_for_pid(records: &[PubkeyRecord], pid: u32) -> Option<&PubkeyRecord> {
    records
        .iter()
        .filter(|record| record.pid == pid)
        .max_by_key(|record| record.timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_record() {
        let record = PubkeyRecord::parse("12345\t67890\t1:abc\t1704067200").unwrap();
        assert_eq!(
            record,
            PubkeyRecord {
                pid: 12345,
                window_id: Some(67890),
                pubkey: "1:abc".to_string(),
                timestamp: 1704067200,
            }
        );
    }

    #[test]
    fn parses_record_without_window_id() {
        let record = PubkeyRecord::parse("12345\t\t1:abc\t1704067200").unwrap();
        assert_eq!(record.window_id, None);
    }

    #[test]
    fn parses_record_without_timestamp_as_zero() {
        let record = PubkeyRecord::parse("12345\t67890\t1:abc").unwrap();
        assert_eq!(record.timestamp, 0);
    }

    #[test]
    fn rejects_bad_pid_and_field_counts() {
        assert!(matches!(
            PubkeyRecord::parse("abc\t67890\t1:abc"),
            Err(RecordError::InvalidPid { .. })
        ));
        assert!(matches!(
            PubkeyRecord::parse("1:only-two"),
            Err(RecordError::FieldCount { found: 1 })
        ));
        assert!(matches!(
            PubkeyRecord::parse("1\t2\t3\t4\t5"),
            Err(RecordError::FieldCount { found: 5 })
        ));
    }

    #[test]
    fn render_round_trips_and_matches_legacy_shape() {
        let record = PubkeyRecord::parse("12345\t67890\t1:abc\t1704067200").unwrap();
        assert_eq!(record.render(), "12345\t67890\t1:abc\t1704067200");
        assert_eq!(
            PubkeyRecord::parse(&record.render()).unwrap(),
            record,
            "rendered records must stay readable"
        );

        let no_window = PubkeyRecord {
            pid: 7,
            window_id: None,
            pubkey: "1:k".to_string(),
            timestamp: 0,
        };
        assert_eq!(no_window.render(), "7\t\t1:k\t0");
    }

    #[test]
    fn store_parse_reports_malformed_lines_with_numbers() {
        let text = "12345\t67890\t1:abc\t100\n\
                    broken\n\
                    \n\
                    12345\t67890\t1:newer\t200\n\
                    nope\t\t1:x\t1\n";
        let contents = parse_store(text);
        assert_eq!(contents.records.len(), 2);
        assert_eq!(
            contents.malformed,
            vec![
                (2, RecordError::FieldCount { found: 1 }),
                (
                    5,
                    RecordError::InvalidPid {
                        value: "nope".to_string()
                    }
                ),
            ]
        );
    }

    #[test]
    fn latest_for_pid_selects_max_timestamp() {
        let records = vec![
            PubkeyRecord::parse("1\t\t1:old\t10").unwrap(),
            PubkeyRecord::parse("2\t\t1:other\t99").unwrap(),
            PubkeyRecord::parse("1\t\t1:new\t20").unwrap(),
        ];
        assert_eq!(latest_for_pid(&records, 1).unwrap().pubkey, "1:new");
        assert_eq!(latest_for_pid(&records, 2).unwrap().pubkey, "1:other");
        assert!(latest_for_pid(&records, 3).is_none());
    }
}
