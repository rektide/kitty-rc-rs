//! Sanitized environment construction for private fixtures.
//!
//! Ambient kitty/terminal/display entrances are removed before any child is
//! spawned; fixture-specific isolation variables are added explicitly. Every
//! supervised child is spawned with `env_clear()` plus exactly this map, so
//! the cleaning cannot be bypassed by an inherited variable.

use std::collections::BTreeMap;
use std::path::Path;

/// Variables that would select or leak an ambient human endpoint,
/// key, terminal or desktop session. `KITTY_RC_MATRIX_*`/`KITTY_RC_TEST_*`
/// test selectors are also stripped so a nested runner cannot inherit a
/// stale selection.
const STRIPPED_EXACT: &[&str] = &[
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XDG_RUNTIME_DIR",
    "DBUS_SESSION_BUS_ADDRESS",
];

fn is_stripped(key: &str) -> bool {
    STRIPPED_EXACT.contains(&key) || key.starts_with("KITTY_") || key.starts_with("XDG_SESSION")
}

/// A copy of the ambient environment with every ambient terminal, kitty and
/// desktop-session entrance removed. `PATH` and other neutral variables are
/// preserved because spawned servers and fixture children need them.
pub fn sanitized_ambient() -> BTreeMap<String, String> {
    std::env::vars()
        .filter(|(key, _)| !is_stripped(key))
        .collect()
}

/// The sanitized ambient environment plus the private fixture's isolation:
/// a dedicated `XDG_RUNTIME_DIR`, a unique headless Wayland socket name,
/// software rendering and private XDG config/cache/data directories.
///
/// The base map is defensively re-filtered: even if a caller supplies an
/// unsanitized base, no spawned child can inherit an ambient kitty/display
/// entrance.
pub fn fixture_environment(
    base: &BTreeMap<String, String>,
    fixture: &Path,
    wayland_socket: &str,
) -> BTreeMap<String, String> {
    let mut env: BTreeMap<String, String> = base
        .iter()
        .filter(|(key, _)| !is_stripped(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    env.insert(
        "XDG_RUNTIME_DIR".into(),
        fixture.to_string_lossy().into_owned(),
    );
    env.insert("WAYLAND_DISPLAY".into(), wayland_socket.into());
    env.insert("LIBGL_ALWAYS_SOFTWARE".into(), "1".into());
    env.insert(
        "XDG_CONFIG_HOME".into(),
        fixture.join("config").to_string_lossy().into_owned(),
    );
    env.insert(
        "XDG_CACHE_HOME".into(),
        fixture.join("cache").to_string_lossy().into_owned(),
    );
    env.insert(
        "XDG_DATA_HOME".into(),
        fixture.join("data").to_string_lossy().into_owned(),
    );
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect()
    }

    #[test]
    fn ambient_kitty_display_and_bus_entrances_are_removed() {
        let source = map(&[
            ("KITTY_PUBLIC_KEY", "human-key"),
            ("KITTY_PID", "1"),
            ("KITTY_WINDOW_ID", "2"),
            ("KITTY_LISTEN_ON", "human.sock"),
            ("KITTY_RC_PASSWORD", "never-log"),
            ("DISPLAY", ":0"),
            ("WAYLAND_DISPLAY", "human-wayland"),
            ("XDG_RUNTIME_DIR", "/run/user/1000"),
            ("DBUS_SESSION_BUS_ADDRESS", "human-bus"),
            ("PATH", "/usr/bin"),
            ("HOME", "/home/human"),
        ]);
        let cleaned: BTreeMap<String, String> = source
            .iter()
            .filter(|(key, _)| !is_stripped(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(
            cleaned,
            map(&[("PATH", "/usr/bin"), ("HOME", "/home/human")])
        );
    }

    #[test]
    fn sanitized_ambient_never_contains_kitty_or_display_keys() {
        // Exercise the real process environment; the filter must hold even
        // when the test runner itself was started from a terminal.
        let cleaned = sanitized_ambient();
        for key in cleaned.keys() {
            assert!(!is_stripped(key), "sanitized environment retained {key}");
        }
    }

    #[test]
    fn fixture_environment_points_every_xdg_and_display_at_the_fixture() {
        let fixture = Path::new("/tmp/opencode/kitty-rc-fixture-x");
        let poisoned = map(&[
            ("PATH", "/usr/bin"),
            ("KITTY_PUBLIC_KEY", "human-key"),
            ("DISPLAY", ":0"),
        ]);
        let env = fixture_environment(&poisoned, fixture, "wayland-1");
        assert_eq!(env["XDG_RUNTIME_DIR"], "/tmp/opencode/kitty-rc-fixture-x");
        assert_eq!(env["WAYLAND_DISPLAY"], "wayland-1");
        assert_eq!(env["LIBGL_ALWAYS_SOFTWARE"], "1");
        assert_eq!(
            env["XDG_CONFIG_HOME"],
            "/tmp/opencode/kitty-rc-fixture-x/config"
        );
        assert_eq!(
            env["XDG_CACHE_HOME"],
            "/tmp/opencode/kitty-rc-fixture-x/cache"
        );
        assert_eq!(
            env["XDG_DATA_HOME"],
            "/tmp/opencode/kitty-rc-fixture-x/data"
        );
        assert_eq!(env["PATH"], "/usr/bin");
        assert!(!env.contains_key("KITTY_PUBLIC_KEY"));
        assert!(!env.contains_key("DISPLAY"));
    }
}
