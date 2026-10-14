//! Pinned upstream release coordinates and executable identification.

use std::path::{Path, PathBuf};

/// Digests published by GitHub's release API, read 2026-10-14. These are
/// integrity/provenance coordinates, not an independently verified signature.
#[derive(Debug)]
pub struct Release {
    pub version: &'static str,
    pub sha256: &'static str,
    pub source_revision: &'static str,
}

pub const RELEASES: &[Release] = &[
    Release {
        version: "0.43.1",
        sha256: "f2026ef8cec6fd21b58e3ae501f707c2fc922ddb9b3e79685917fc60197449fb",
        source_revision: "7f24dd68c98d8f4712457560f25e8056cce52ec8",
    },
    Release {
        version: "0.49.2",
        sha256: "d573618b911e9c461bd421b96c13c74c7f1cb2f1ac9c327818d4ff84366cf5c6",
        source_revision: "b2cb871254fbdac19a085188813c8dfb258b95cd",
    },
];

pub fn release(version: &str) -> Option<&'static Release> {
    RELEASES.iter().find(|entry| entry.version == version)
}

/// Ignored project scratch holding acquired releases, provenance and matrix
/// fixtures, relative to the workspace root.
pub const SCRATCH_RELATIVE: &str = ".test-agent/supported-kitty";

pub fn scratch(root: &Path) -> PathBuf {
    root.join(SCRATCH_RELATIVE)
}

pub fn executable_in_scratch(root: &Path, version: &str) -> PathBuf {
    scratch(root).join(version).join("app/bin/kitty")
}

pub fn archive_url(version: &str) -> String {
    format!(
        "https://github.com/kovidgoyal/kitty/releases/download/v{version}/kitty-{version}-x86_64.txz"
    )
}

/// Parse a `kitty --version` report into `(version, [major, minor, patch])`.
///
/// Mirrors the previous `kitty (\d+\.\d+\.\d+)\b` check: development strings
/// such as `kitty 0.50dev` are rejected, not guessed into a release.
pub fn parse_version(reported: &str) -> Option<([u32; 3], &str)> {
    let rest = reported.trim().strip_prefix("kitty ")?;
    let end = rest
        .char_indices()
        .find(|(_, character)| !matches!(character, '0'..='9' | '.'))
        .map(|(index, _)| index)
        .unwrap_or(rest.len());
    let triple = &rest[..end];
    // A word character directly after the digits (0.50dev) is not a release.
    if end < rest.len() {
        let next = rest[end..].chars().next().unwrap();
        if next.is_ascii_alphanumeric() || next == '_' {
            return None;
        }
    }
    let parts: Vec<_> = triple
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<_, _>>()
        .ok()?;
    if parts.len() != 3 {
        return None;
    }
    let numbers = [parts[0], parts[1], parts[2]];
    if numbers.iter().all(|part| *part == 0) {
        return None;
    }
    Some((numbers, triple))
}

/// Hex SHA-256 of a file, streamed.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exact_release_reports() {
        let text = "kitty 0.48.2 created by Kovid Goyal";
        let (numbers, version) = parse_version(text).unwrap();
        assert_eq!(numbers, [0, 48, 2]);
        assert_eq!(version, "0.48.2");
        assert_eq!(
            parse_version("kitty 0.43.1 created by Kovid Goyal")
                .unwrap()
                .0,
            [0, 43, 1]
        );
    }

    #[test]
    fn rejects_development_and_malformed_reports() {
        assert!(parse_version("kitty 0.50dev").is_none());
        assert!(parse_version("kitty 0.50.1dev").is_none());
        assert!(parse_version("kitty 0.50").is_none());
        assert!(parse_version("not kitty 0.48.2").is_none());
        assert!(parse_version("kitty 0.48.2x").is_none());
    }

    #[test]
    fn scratch_layout_matches_the_acquired_releases() {
        let root = Path::new("/repo");
        assert_eq!(
            scratch(root),
            Path::new("/repo/.test-agent/supported-kitty")
        );
        assert_eq!(
            executable_in_scratch(root, "0.43.1"),
            Path::new("/repo/.test-agent/supported-kitty/0.43.1/app/bin/kitty")
        );
        assert_eq!(
            archive_url("0.49.2"),
            "https://github.com/kovidgoyal/kitty/releases/download/v0.49.2/kitty-0.49.2-x86_64.txz"
        );
    }

    #[test]
    fn sha256_file_matches_a_known_digest() {
        let file = std::env::temp_dir().join("kitty-rc-verify-sha256-unit");
        std::fs::write(&file, b"abc").unwrap();
        assert_eq!(
            sha256_file(&file).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_file(&file).unwrap();
    }
}
