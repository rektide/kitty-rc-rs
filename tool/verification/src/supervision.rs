//! Bounded ownership of supervised fixture processes.
//!
//! Every spawned child becomes its own process-group leader, so termination
//! reaches the whole owned tree (weston's compositor children, kitty's
//! terminal child) rather than only the direct subprocess. Normal teardown
//! sends SIGTERM with a grace period, the deadline/Ctrl-C path sends SIGINT
//! (letting kitty's own handlers run) and both escalate to SIGKILL. If a
//! future is cancelled or panics before explicit teardown, `kill_on_drop`
//! plus a group-kill `Drop` backstop guarantees no owned child survives.

use std::collections::BTreeMap;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Child;

#[derive(Debug, Error)]
pub enum SupervisionError {
    #[error("spawning private {process} failed")]
    Spawn {
        process: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("waiting for private {process} failed")]
    Wait {
        process: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("private {process} exited with {status}")]
    Exited {
        process: &'static str,
        status: String,
    },
}

/// A supervised, owned fixture process.
pub struct Supervised {
    process: &'static str,
    child: Child,
    pgid: i32,
    log: PathBuf,
}

impl Supervised {
    /// Spawn `program` in its own process group with a fully replaced
    /// environment, appending combined output to `log`.
    pub fn spawn(
        process: &'static str,
        program: &Path,
        args: &[String],
        env: &BTreeMap<String, String>,
        cwd: Option<&Path>,
        log: &Path,
    ) -> Result<Self, SupervisionError> {
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .map_err(|source| SupervisionError::Spawn { process, source })?;
        let mut std_command = std::process::Command::new(program);
        std_command
            .args(args)
            .env_clear()
            .envs(env)
            .stdout(Stdio::from(log_file.try_clone().map_err(|source| {
                SupervisionError::Spawn { process, source }
            })?))
            .stderr(Stdio::from(log_file));
        if let Some(directory) = cwd {
            std_command.current_dir(directory);
        }
        std_command.process_group(0);
        let mut command = tokio::process::Command::from(std_command);
        command.kill_on_drop(true);
        let child = command
            .spawn()
            .map_err(|source| SupervisionError::Spawn { process, source })?;
        let pgid = child.id().expect("spawned child has a pid") as i32;
        Ok(Self {
            process,
            child,
            pgid,
            log: log.to_owned(),
        })
    }

    pub fn process_name(&self) -> &'static str {
        self.process
    }

    pub fn pgid(&self) -> i32 {
        self.pgid
    }

    pub fn log_path(&self) -> &Path {
        &self.log
    }

    /// Non-blocking liveness check.
    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus, SupervisionError> {
        self.child
            .wait()
            .await
            .map_err(|source| SupervisionError::Wait {
                process: self.process,
                source,
            })
    }

    fn signal_group(&self, signal: libc::c_int) {
        // SAFETY: killpg only delivers a signal to the named owned group.
        unsafe { libc::killpg(self.pgid, signal) };
    }

    /// Graceful teardown: SIGTERM the owned group, then SIGKILL after `grace`.
    pub async fn shutdown(mut self, grace: Duration) -> Result<(), SupervisionError> {
        self.signal_group(libc::SIGTERM);
        match tokio::time::timeout(grace, self.child.wait()).await {
            Ok(status) => status.map(|_| ()).map_err(|source| SupervisionError::Wait {
                process: self.process,
                source,
            }),
            Err(_) => {
                self.signal_group(libc::SIGKILL);
                self.child
                    .wait()
                    .await
                    .map(|_| ())
                    .map_err(|source| SupervisionError::Wait {
                        process: self.process,
                        source,
                    })
            }
        }
    }

    /// Deadline/Ctrl-C teardown: SIGINT lets lifecycle owners run their own
    /// final handlers, escalating to SIGKILL after `grace`.
    pub async fn interrupt(mut self, grace: Duration) -> Result<(), SupervisionError> {
        self.signal_group(libc::SIGINT);
        match tokio::time::timeout(grace, self.child.wait()).await {
            Ok(status) => status.map(|_| ()).map_err(|source| SupervisionError::Wait {
                process: self.process,
                source,
            }),
            Err(_) => {
                self.signal_group(libc::SIGKILL);
                self.child
                    .wait()
                    .await
                    .map(|_| ())
                    .map_err(|source| SupervisionError::Wait {
                        process: self.process,
                        source,
                    })
            }
        }
    }
}

impl Drop for Supervised {
    fn drop(&mut self) {
        // Backstop for cancellation/panic paths: SIGKILL the whole owned
        // group synchronously. The awaited methods above are the graceful
        // path; this only fires when they never ran.
        self.signal_group(libc::SIGKILL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown_terminates_a_sleeping_group_leader() {
        let scratch = std::env::temp_dir().join("kitty-rc-verify-supervision-unit");
        std::fs::create_dir_all(&scratch).unwrap();
        let log = scratch.join("process.log");
        let sleep = Supervised::spawn(
            "sleep-child",
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 300".into()],
            &BTreeMap::new(),
            None,
            &log,
        )
        .unwrap();
        let pid = sleep.pgid;
        sleep.shutdown(Duration::from_secs(5)).await.unwrap();
        // The group leader is gone once shutdown returns.
        // SAFETY: checking existence of the owned pid's process.
        let remains = unsafe { libc::killpg(pid, 0) == 0 };
        assert!(
            !remains,
            "owned process group {pid} still alive after shutdown"
        );
        std::fs::remove_dir_all(&scratch).ok();
    }
}
