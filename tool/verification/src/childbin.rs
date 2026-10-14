//! Shared vocabulary between the runner and the `kitty-rc-verify-child`
//! binary that private kitty fixtures launch inside their terminal window.
//!
//! The child is a real process kitty parents onto the fixture window, so the
//! examples can observe startup markers, a published `KITTY_PUBLIC_KEY`,
//! long-running work with sentinels and raw stream journeys — without any
//! Python `-c` strings. Both sides of this contract live in this crate; the
//! helpers below construct the argument lists the child parses
//! (see `src/child.rs`).

use std::path::Path;

/// `kitty-rc-verify-child key <KEY-FILE> --marker <TEXT> --sleep-secs <N>`:
/// publishes the fixture's `KITTY_PUBLIC_KEY`, prints `marker` (which
/// `get-text` journeys later observe on screen) and stays alive for the
/// fixture lifetime.
pub fn key_publication(
    child: &Path,
    key_file: &Path,
    marker: &str,
    sleep_secs: u64,
) -> Vec<String> {
    vec![
        child.to_string_lossy().into_owned(),
        "key".into(),
        key_file.to_string_lossy().into_owned(),
        "--marker".into(),
        marker.into(),
        "--sleep-secs".into(),
        sleep_secs.to_string(),
    ]
}

/// `kitty-rc-verify-child sentinel <PATH> --sleep-ms <N> --exit <CODE>`:
/// writes `started` to `PATH`, sleeps, then exits with `CODE`. Used by
/// launch-wait journeys that prove short work finishes while this long
/// operation is still running.
pub fn sentinel(child: &Path, path: &Path, sleep_ms: u64, exit_code: i32) -> Vec<String> {
    vec![
        child.to_string_lossy().into_owned(),
        "sentinel".into(),
        path.to_string_lossy().into_owned(),
        "--sleep-ms".into(),
        sleep_ms.to_string(),
        "--exit".into(),
        exit_code.to_string(),
    ]
}

/// `kitty-rc-verify-child stream --stderr-hex <HEX> --exit <CODE>`: copies
/// stdin to stdout byte-for-byte, writes the fixed stderr bytes and exits
/// with `CODE`. Raw upload journeys check exact binary round trips.
pub fn stream(child: &Path, stderr_hex: &str, exit_code: i32) -> Vec<String> {
    vec![
        child.to_string_lossy().into_owned(),
        "stream".into(),
        "--stderr-hex".into(),
        stderr_hex.into(),
        "--exit".into(),
        exit_code.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argument_lists_round_trip_through_the_child_cli_shape() {
        let child = Path::new("/bin/kitty-rc-verify-child");
        assert_eq!(
            key_publication(child, Path::new("/f/key"), "marker text", 120),
            vec![
                "/bin/kitty-rc-verify-child",
                "key",
                "/f/key",
                "--marker",
                "marker text",
                "--sleep-secs",
                "120",
            ]
        );
        assert_eq!(
            sentinel(child, Path::new("/f/started"), 1500, 7),
            vec![
                "/bin/kitty-rc-verify-child",
                "sentinel",
                "/f/started",
                "--sleep-ms",
                "1500",
                "--exit",
                "7",
            ]
        );
        assert_eq!(
            stream(child, "00ff80", 7),
            vec![
                "/bin/kitty-rc-verify-child",
                "stream",
                "--stderr-hex",
                "00ff80",
                "--exit",
                "7"
            ]
        );
    }
}
