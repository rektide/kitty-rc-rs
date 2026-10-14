//! JSON evidence records for the supported-version matrix. Field names follow
//! the retired Python runner's report so pre/post-conversion witnesses stay
//! comparable; the `runner` fields now name the Rust binary.

use crate::Protection;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize)]
pub struct MatrixReport {
    pub at: String,
    pub request_envelope: [u32; 3],
    pub finite_only: bool,
    pub cli_smoke: bool,
    pub rustc: String,
    pub examples: BTreeMap<String, String>,
    pub runner: String,
    pub runner_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_sha256: Option<String>,
    pub servers: Vec<ServerRecord>,
}

impl MatrixReport {
    pub fn new(finite_only: bool, cli_smoke: bool, rustc: String, runner: &Path) -> Self {
        Self {
            at: now_iso8601(),
            request_envelope: [0, 43, 1],
            finite_only,
            cli_smoke,
            rustc,
            examples: BTreeMap::new(),
            runner: runner.display().to_string(),
            runner_sha256: crate::release::sha256_file(runner).unwrap_or_default(),
            cli_sha256: None,
            servers: Vec::new(),
        }
    }

    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap() + "\n")
    }
}

#[derive(Serialize)]
pub struct ServerRecord {
    pub executable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
    pub status: ServerStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub profiles: Vec<ProfileRecord>,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Ready,
    Blocked,
}

#[derive(Serialize)]
pub struct ProfileRecord {
    pub protection: Protection,
    pub status: ProfileStatus,
    pub records: Vec<serde_json::Value>,
    pub output: String,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    Pass,
    Fail,
    Blocked,
}

/// Parse the newline-delimited JSON case records an example emits. Records
/// must carry a `case` or `harness` key; other lines are diagnostics.
pub fn parse_case_records(output: &str) -> Vec<serde_json::Value> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|record| record.get("case").is_some() || record.get("harness").is_some())
        .collect()
}

fn now_iso8601() -> String {
    // RFC 3339 UTC from the system clock; no external time dependency.
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_seconds = since_epoch.as_secs();
    let days = total_seconds / 86_400;
    let seconds_of_day = total_seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds_of_day / 3600,
        (seconds_of_day % 3600) / 60,
        seconds_of_day % 60
    )
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_records_are_extracted_and_diagnostics_ignored() {
        let output = "setting up\n{\"case\":\"direct-reuse\",\"status\":\"pass\"}\nnoise {\"x\":1}\n{\"harness\":\"conformance\",\"status\":\"pass\"}\n";
        let records = parse_case_records(output);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["case"], "direct-reuse");
        assert_eq!(records[1]["harness"], "conformance");
    }

    #[test]
    fn timestamp_is_rfc3339_utc_shaped() {
        let stamp = now_iso8601();
        // e.g. 2026-10-14T12:34:56Z
        assert_eq!(stamp.len(), 20);
        assert!(stamp.ends_with('Z'));
        assert_eq!(&stamp[4..5], "-");
        assert_eq!(&stamp[10..11], "T");
    }

    #[test]
    fn civil_dates_round_trip_known_days() {
        // Verified day numbers since the epoch.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_740), (2026, 10, 14));
    }
}
