//! The private headless Weston/kitty fixture.
//!
//! One [`Fixture`] owns: a unique fixture directory (`kitty-rc-fixture-*`)
//! holding `kitty.conf`, private XDG config/cache/data, the fixture-child key
//! publication, the RC unix socket and a combined process log; one headless
//! Weston compositor in its own process group; one kitty server in its own
//! process group running the fixture child. Ambient kitty/display entrances
//! are stripped from every spawned process. Startup is bounded (the 5-second
//! predecessor deadline was insufficient once; default here is 30 seconds)
//! and teardown is guaranteed on every path via [`crate::supervision`].

use crate::childbin;
use crate::env::{fixture_environment, sanitized_ambient};
use crate::supervision::{Supervised, SupervisionError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

/// Directory-name prefix the conformance examples recognize as private.
pub const FIXTURE_PREFIX: &str = "kitty-rc-fixture-";
/// Legacy prefixes from the retired Python runners, still recognized by the
/// examples so evidence recorded before the conversion remains readable.
pub const LEGACY_PREFIXES: &[&str] = &["kitty-rc-live-", "kitty-private-upload-"];

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("private fixture setup failed: {context}")]
    Setup {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("private {process} failed before the fixture was ready (log follows):\n{log}")]
    ProcessFailed { process: &'static str, log: String },
    #[error("private fixture was not ready within {timeout:?} (log follows):\n{log}")]
    StartupTimeout { timeout: Duration, log: String },
    #[error("teardown of private {process} failed")]
    Teardown {
        process: &'static str,
        #[source]
        source: SupervisionError,
    },
}

pub struct FixtureOptions {
    /// Explicit kitty executable; never discovered from an ambient terminal.
    pub kitty: PathBuf,
    pub weston: PathBuf,
    /// The `kitty-rc-verify-child` binary kitty launches inside the window.
    pub child: PathBuf,
    /// Fixture-local test password; never a human credential.
    pub password: String,
    /// Select `allow_remote_control yes` instead of password protection.
    pub plain: bool,
    /// Append the matrix runner's convenience options
    /// (`dynamic_background_opacity`, `confirm_os_window_close 0`).
    pub augment_config: bool,
    /// Marker the child prints and `get-text` journeys expect on screen.
    pub marker: String,
    /// Bounded startup deadline; the predecessor's 5 s failed once.
    pub startup_timeout: Duration,
    /// How long the publication child stays alive.
    pub child_sleep: Duration,
    /// Injectable base environment (tests); defaults to the sanitized ambient.
    pub base_env: Option<BTreeMap<String, String>>,
}

impl Default for FixtureOptions {
    fn default() -> Self {
        Self {
            kitty: PathBuf::from("kitty"),
            weston: PathBuf::from("weston"),
            child: PathBuf::from("kitty-rc-verify-child"),
            password: "test-secret".into(),
            plain: false,
            augment_config: false,
            marker: "kitty-rc private fixture".into(),
            startup_timeout: Duration::from_secs(30),
            child_sleep: Duration::from_secs(120),
            base_env: None,
        }
    }
}

pub struct Fixture {
    pub dir: PathBuf,
    pub socket: PathBuf,
    pub key: String,
    pub password: String,
    pub plain: bool,
    wayland: String,
    weston: Option<Supervised>,
    kitty: Option<Supervised>,
    log: PathBuf,
    base_env: BTreeMap<String, String>,
}

impl Fixture {
    /// Environment for processes participating in this fixture: the sanitized
    /// ambient environment plus this fixture's isolation variables.
    pub fn environment(&self) -> BTreeMap<String, String> {
        fixture_environment(&self.base_env, &self.dir, &self.wayland)
    }

    /// Combined process log, password redacted.
    pub fn log_text(&self) -> String {
        crate::redact_password(
            &std::fs::read_to_string(&self.log).unwrap_or_default(),
            &self.password,
        )
    }

    /// Tear the fixture's own processes down and consume it. On the clean
    /// path the private fixture directory is removed; failure paths keep it
    /// (with `process.log`) for diagnosis.
    pub async fn finish(mut self) -> Result<(), FixtureError> {
        let mut teardown_ok = true;
        if let Some(kitty) = self.kitty.take()
            && kitty.shutdown(Duration::from_secs(5)).await.is_err()
        {
            teardown_ok = false;
        }
        if let Some(weston) = self.weston.take()
            && weston.shutdown(Duration::from_secs(5)).await.is_err()
        {
            teardown_ok = false;
        }
        if teardown_ok {
            std::fs::remove_dir_all(&self.dir).ok();
            Ok(())
        } else {
            Err(FixtureError::Setup {
                context: "at least one owned process failed to tear down cleanly".into(),
                source: Box::new(std::io::Error::other("teardown failure")),
            })
        }
    }

    /// Deadline teardown: SIGINT first so lifecycle owners can run final
    /// handlers, SIGKILL after `grace`.
    pub async fn interrupt(mut self, grace: Duration) -> Result<(), FixtureError> {
        if let Some(kitty) = self.kitty.take() {
            kitty
                .interrupt(grace)
                .await
                .map_err(|source| FixtureError::Teardown {
                    process: "kitty",
                    source,
                })?;
        }
        if let Some(weston) = self.weston.take() {
            weston
                .interrupt(grace)
                .await
                .map_err(|source| FixtureError::Teardown {
                    process: "weston",
                    source,
                })?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Fixture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Fixture")
            .field("dir", &self.dir)
            .field("socket", &self.socket)
            .field("key", &"<redacted>")
            .field("password", &"<redacted>")
            .field("plain", &self.plain)
            .field(
                "weston",
                &self.weston.as_ref().map(|process| process.process_name()),
            )
            .field(
                "kitty",
                &self.kitty.as_ref().map(|process| process.process_name()),
            )
            .finish()
    }
}

fn unique_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() % 1_000_000)
        .unwrap_or_default();
    format!(
        "{}-{}-{millis}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

/// Compose the private kitty.conf for the selected protection profile.
pub fn kitty_config(password: &str, plain: bool, augment: bool) -> String {
    let mut config = String::from(if plain {
        "allow_remote_control yes\n"
    } else {
        "allow_remote_control password\n"
    });
    config.push_str(&format!("remote_control_password {password}\n"));
    config.push_str("linux_display_server wayland\n");
    if augment {
        config.push_str("dynamic_background_opacity yes\nconfirm_os_window_close 0\n");
    }
    config
}

/// Start a private fixture under `parent` (which must exist).
pub async fn start(options: FixtureOptions, parent: &Path) -> Result<Fixture, FixtureError> {
    let suffix = unique_suffix();
    let dir = parent.join(format!("{FIXTURE_PREFIX}{suffix}"));
    let log = dir.join("process.log");
    std::fs::create_dir_all(&dir).map_err(|source| FixtureError::Setup {
        context: format!("creating {}", dir.display()),
        source: Box::new(source),
    })?;
    for name in ["config", "cache", "data"] {
        std::fs::create_dir_all(dir.join(name)).map_err(|source| FixtureError::Setup {
            context: format!("creating {name} dir"),
            source: Box::new(source),
        })?;
    }
    std::fs::write(
        dir.join("kitty.conf"),
        kitty_config(&options.password, options.plain, options.augment_config),
    )
    .map_err(|source| FixtureError::Setup {
        context: "writing kitty.conf".into(),
        source: Box::new(source),
    })?;
    // The checked-in independent no-ui kitten fixture (an upstream Python
    // kitten; see lib.rs). Copied verbatim into the private fixture.
    std::fs::write(dir.join("no-ui.py"), include_str!("../fixtures/no_ui.py")).map_err(
        |source| FixtureError::Setup {
            context: "writing no-ui.py".into(),
            source: Box::new(source),
        },
    )?;

    let base_env = options.base_env.clone().unwrap_or_else(sanitized_ambient);
    // Socket paths must stay well under sockaddr_un's 108-byte limit, so the
    // Wayland socket name stays short; the fixture directory prefix carries
    // the recognizable private-fixture marker.
    let wayland_name = format!("wl-{suffix}");
    let env = fixture_environment(&base_env, &dir, &wayland_name);
    let socket = dir.join("rc.sock");
    let key_file = dir.join("key");

    let weston_args: Vec<String> = [
        "--backend=headless",
        "--renderer=pixman",
        "--fake-seat",
        "--socket",
        &wayland_name,
        "--no-config",
    ]
    .iter()
    .map(|value| (*value).into())
    .collect();
    let mut weston = Supervised::spawn("weston", &options.weston, &weston_args, &env, None, &log)
        .map_err(|source| FixtureError::Setup {
        context: "spawning private weston".into(),
        source: Box::new(source),
    })?;

    let weston_socket = dir.join(&wayland_name);
    let deadline = tokio::time::Instant::now() + options.startup_timeout;
    loop {
        if weston_socket.exists() {
            break;
        }
        if weston.has_exited() {
            return Err(FixtureError::ProcessFailed {
                process: "weston",
                log: crate::redact_password(
                    &std::fs::read_to_string(&log).unwrap_or_default(),
                    &options.password,
                ),
            });
        }
        if tokio::time::Instant::now() >= deadline {
            weston.shutdown(Duration::from_secs(5)).await.ok();
            return Err(FixtureError::StartupTimeout {
                timeout: options.startup_timeout,
                log: crate::redact_password(
                    &std::fs::read_to_string(&log).unwrap_or_default(),
                    &options.password,
                ),
            });
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let child_args = childbin::key_publication(
        &options.child,
        &key_file,
        &options.marker,
        options.child_sleep.as_secs(),
    );
    let mut kitty_args: Vec<String> = vec![
        "--config".into(),
        dir.join("kitty.conf").to_string_lossy().into_owned(),
        "--listen-on".into(),
        format!("unix:{}", socket.display()),
    ];
    kitty_args.extend(child_args);
    let mut kitty = match Supervised::spawn("kitty", &options.kitty, &kitty_args, &env, None, &log)
    {
        Ok(kitty) => kitty,
        Err(source) => {
            weston.shutdown(Duration::from_secs(5)).await.ok();
            return Err(FixtureError::Setup {
                context: "spawning private kitty".into(),
                source: Box::new(source),
            });
        }
    };

    loop {
        if key_file.exists() && socket.exists() {
            break;
        }
        if kitty.has_exited() {
            let log = crate::redact_password(
                &std::fs::read_to_string(&log).unwrap_or_default(),
                &options.password,
            );
            weston.shutdown(Duration::from_secs(5)).await.ok();
            return Err(FixtureError::ProcessFailed {
                process: "kitty",
                log,
            });
        }
        if tokio::time::Instant::now() >= deadline {
            let log = crate::redact_password(
                &std::fs::read_to_string(&log).unwrap_or_default(),
                &options.password,
            );
            kitty.shutdown(Duration::from_secs(5)).await.ok();
            weston.shutdown(Duration::from_secs(5)).await.ok();
            return Err(FixtureError::StartupTimeout {
                timeout: options.startup_timeout,
                log,
            });
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let key = std::fs::read_to_string(&key_file)
        .map_err(|source| FixtureError::Setup {
            context: "reading published fixture key".into(),
            source: Box::new(source),
        })?
        .trim()
        .to_owned();

    Ok(Fixture {
        dir,
        socket,
        key,
        password: options.password,
        plain: options.plain,
        wayland: wayland_name,
        weston: Some(weston),
        kitty: Some(kitty),
        log,
        base_env,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_encodes_the_selected_protection_profile() {
        assert!(
            kitty_config("test-secret", false, false).contains("allow_remote_control password\n")
        );
        assert!(
            kitty_config("test-secret", false, false)
                .contains("remote_control_password test-secret\n")
        );
        assert!(kitty_config("test-secret", true, false).contains("allow_remote_control yes\n"));
        let augmented = kitty_config("test-secret", false, true);
        assert!(augmented.contains("dynamic_background_opacity yes\n"));
        assert!(augmented.contains("confirm_os_window_close 0\n"));
        assert!(!kitty_config("test-secret", false, false).contains("dynamic_background_opacity"));
    }
}
