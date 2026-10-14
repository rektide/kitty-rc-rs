//! `kitty-pubkey-db`: explicit management of persisted kitty server public keys
//! used for password-authenticated remote control.
//!
//! Behavior preserved from the legacy self-contained binary, with two
//! deliberate changes: record parsing/rendering comes from the shared
//! `kitty-rc-key-store` library (so future credential composition and the CLI
//! agree on the format), and the legacy **automatic cleanup-on-add epoch
//! machinery is gone** — cleanup is an explicit command, never a side effect.
//! Existing store files are read as-is; malformed lines are reported, not
//! erased.

use crate::Error;
use clap::{Parser, Subcommand};
use kitty_rc_key_store::{PubkeyRecord, latest_for_pid, storage};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(name = "kitty-pubkey-db")]
#[command(about = "Manage kitty public keys for password authentication", long_about = None)]
pub struct Cli {
    /// Explicit store path; default is $XDG_STATE_HOME/kitty/pubkey.tsv
    #[arg(long, global = true)]
    store: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the database
    Init,
    /// Add a public key entry (all parameters optional, reads from env vars)
    Add {
        /// PID of kitty instance (reads from KITTY_PID if not provided)
        #[arg(long)]
        pid: Option<u32>,
        /// Base85 encoded public key (reads from KITTY_PUBLIC_KEY if not provided)
        #[arg(long)]
        pubkey: Option<String>,
        /// Window ID (optional, reads from KITTY_WINDOW_ID if not provided)
        #[arg(long)]
        window_id: Option<u32>,
    },
    /// Clean up stale entries (explicit; keeps records for running processes)
    Cleanup {
        /// Assert that all non-CLI publishers are stopped during maintenance
        #[arg(long, required = true)]
        quiescent: bool,
    },
    /// Get the newest public key recorded for a PID
    Get {
        /// PID of the kitty instance
        pid: u32,
    },
}

pub fn run(cli: Cli) -> Result<(), Error> {
    let path = match cli.store {
        Some(path) => path,
        None => db_path()?,
    };
    match cli.command {
        Commands::Init => init(&path)?,
        Commands::Add {
            pid,
            window_id,
            pubkey,
        } => add(&path, pid, window_id, pubkey)?,
        Commands::Cleanup { quiescent: _ } => cleanup(&path)?,
        Commands::Get { pid } => get(&path, pid)?,
    }

    Ok(())
}

fn init(db_path: &Path) -> Result<(), Error> {
    fs::create_dir_all(parent(db_path)?)?;
    let _lock = StoreLock::acquire(db_path)?;
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(db_path)
    {
        Ok(_) => println!("Created database: {}", db_path.display()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            storage::read(db_path, MAX_STORE)?;
            println!("Database already exists: {}", db_path.display());
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

fn add(
    path: &Path,
    pid: Option<u32>,
    window_id: Option<u32>,
    pubkey: Option<String>,
) -> Result<(), Error> {
    let pid = match pid {
        Some(pid) => Some(pid),
        None => env_u32("KITTY_PID")?,
    }
    .ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "PID must be provided via --pid or KITTY_PID env var",
        )
    })?;

    let pubkey = match pubkey {
        Some(pubkey) => Some(pubkey),
        None => env_text("KITTY_PUBLIC_KEY")?,
    }
    .ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Public key must be provided via --pubkey or KITTY_PUBLIC_KEY env var",
        )
    })?;

    let window_id = match window_id {
        Some(id) => Some(id),
        None => env_u32("KITTY_WINDOW_ID")?,
    };

    let record = PubkeyRecord {
        pid,
        window_id,
        pubkey,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };

    fs::create_dir_all(parent(path)?)?;
    let _lock = StoreLock::acquire(path)?;
    storage::append(path, &record)?;

    Ok(())
}

fn cleanup(db_path: &Path) -> Result<(), Error> {
    fs::create_dir_all(parent(db_path)?)?;
    let _lock = StoreLock::acquire(db_path)?;
    let Some(initial) = storage::read(db_path, MAX_STORE)? else {
        println!("Database does not exist");
        return Ok(());
    };
    reject_malformed(&initial)?;
    let before = fs::read(db_path)?;
    // Also validate the exact replacement basis, not a previous separate read.
    let text = std::str::from_utf8(&before)?;
    let contents = kitty_rc_key_store::parse_store(text);
    reject_malformed(&contents)?;
    #[cfg(target_os = "linux")]
    fs::metadata("/proc/self").map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("cleanup requires mounted readable procfs: {error}"),
        )
    })?;

    let alive: Vec<&PubkeyRecord> = contents
        .records
        .iter()
        .map(|record| is_process_running(record.pid).map(|running| running.then_some(record)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    let removed = contents.records.len() - alive.len();

    // Preserve surviving legacy record bytes (including CRLF/no final LF).
    let replacement: Vec<u8> = before
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| {
            let text = std::str::from_utf8(line).expect("shared read validated UTF-8");
            kitty_rc_key_store::parse_store(text)
                .records
                .first()
                .is_none_or(|record| alive.iter().any(|alive| alive.pid == record.pid))
        })
        .flatten()
        .copied()
        .collect();
    let temporary = db_path.with_extension("tsv.cleanup");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> io::Result<()> {
        file.write_all(&replacement)?;
        file.set_permissions(fs::metadata(db_path)?.permissions())?;
        file.sync_all()?;
        if fs::read(db_path)? != before {
            return Err(io::Error::other(
                "store changed during cleanup; stopped without replacing it",
            ));
        }
        fs::rename(&temporary, db_path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;

    println!(
        "Cleanup complete: kept {} of {} entries (removed {} stale)",
        alive.len(),
        contents.records.len(),
        removed
    );

    Ok(())
}

fn get(db_path: &Path, pid: u32) -> Result<(), Error> {
    let contents =
        storage::read(db_path, MAX_STORE)?.ok_or("database does not exist; run init or add")?;
    reject_malformed(&contents)?;

    if let Some(record) = latest_for_pid(&contents.records, pid) {
        println!("{}", record.pubkey);
    } else {
        return Err(format!("no public key found for PID {pid}").into());
    }

    Ok(())
}

fn parent(path: &Path) -> Result<&Path, Error> {
    Ok(path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new(".")))
}
fn db_path() -> Result<PathBuf, Error> {
    let state = match std::env::var_os("XDG_STATE_HOME") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => {
            PathBuf::from(std::env::var_os("HOME").ok_or("select --store or HOME/XDG_STATE_HOME")?)
                .join(".local/state")
        }
    };
    Ok(state.join("kitty/pubkey.tsv"))
}
fn env_text(name: &str) -> Result<Option<String>, Error> {
    std::env::var_os(name)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| format!("selected {name} is not Unicode").into())
        })
        .transpose()
}
fn env_u32(name: &str) -> Result<Option<u32>, Error> {
    env_text(name)?
        .map(|value| {
            value
                .parse()
                .map_err(|_| format!("selected {name} must be a u32").into())
        })
        .transpose()
}

fn is_process_running(pid: u32) -> io::Result<bool> {
    #[cfg(target_os = "linux")]
    {
        match fs::metadata(format!("/proc/{pid}")) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "cleanup process inspection is implemented only for Linux procfs",
        ))
    }
}

const MAX_STORE: usize = 16 * 1024 * 1024;

fn reject_malformed(contents: &kitty_rc_key_store::StoreContents) -> Result<(), Error> {
    if contents.malformed.is_empty() {
        return Ok(());
    }
    for (line, error) in &contents.malformed {
        eprintln!("malformed record at line {line}: {error}");
    }
    Err("malformed store; refusing operation without changing its bytes".into())
}

/// Cooperative CLI-writer lock. No stale-lock deletion or automatic cleanup.
struct StoreLock(PathBuf);
impl StoreLock {
    fn acquire(path: &Path) -> io::Result<Self> {
        let path = path.with_extension("tsv.lock");
        let mut file = OpenOptions::new().write(true).create_new(true).open(&path)
            .map_err(|error| io::Error::new(error.kind(), format!("acquire store lock {path:?}: {error}; stop competing writers; inspect stale locks deliberately")))?;
        let lock = Self(path);
        writeln!(file, "{}", std::process::id())?;
        Ok(lock)
    }
}
impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
