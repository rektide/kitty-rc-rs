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

use clap::{Parser, Subcommand};
use kitty_rc_key_store::{PubkeyRecord, latest_for_pid, storage};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(name = "kitty-pubkey-db")]
#[command(about = "Manage kitty public keys for password authentication", long_about = None)]
struct Cli {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init()?,
        Commands::Add {
            pid,
            window_id,
            pubkey,
        } => add(pid, window_id, pubkey)?,
        Commands::Cleanup { quiescent: _ } => cleanup()?,
        Commands::Get { pid } => get(pid)?,
    }

    Ok(())
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    let db_dir = db_dir()?;
    fs::create_dir_all(&db_dir)?;

    let db_path = db_path()?;
    let _lock = StoreLock::acquire(&db_path)?;
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&db_path)
    {
        Ok(_) => println!("Created database: {}", db_path.display()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            storage::read(&db_path, MAX_STORE)?;
            println!("Database already exists: {}", db_path.display());
        }
        Err(error) => return Err(error.into()),
    }

    println!("\nAdd this to your ~/.zshrc:");
    println!("  # Record kitty public key when shell starts");
    println!("  if [[ -n \"$KITTY_PUBLIC_KEY\" && -n \"$KITTY_PID\" ]]; then");
    print!("      kitty-pubkey-db add &");
    println!();
    println!("      disown");
    println!("  fi");

    Ok(())
}

fn add(
    pid: Option<u32>,
    window_id: Option<u32>,
    pubkey: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pid = pid
        .or_else(|| std::env::var("KITTY_PID").ok().and_then(|s| s.parse().ok()))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "PID must be provided via --pid or KITTY_PID env var",
            )
        })?;

    let pubkey = pubkey
        .or_else(|| std::env::var("KITTY_PUBLIC_KEY").ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Public key must be provided via --pubkey or KITTY_PUBLIC_KEY env var",
            )
        })?;

    let window_id = window_id.or_else(|| {
        std::env::var("KITTY_WINDOW_ID")
            .ok()
            .and_then(|s| s.parse().ok())
    });

    let record = PubkeyRecord {
        pid,
        window_id,
        pubkey,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };

    let path = db_path()?;
    fs::create_dir_all(path.parent().ok_or("store has no parent")?)?;
    let _lock = StoreLock::acquire(&path)?;
    storage::append(&path, &record)?;

    Ok(())
}

fn cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = db_path()?;

    fs::create_dir_all(db_dir()?)?;
    let _lock = StoreLock::acquire(&db_path)?;
    let Some(contents) = storage::read(&db_path, MAX_STORE)? else {
        println!("Database does not exist");
        return Ok(());
    };
    reject_malformed(&contents)?;
    let before = fs::read(&db_path)?;

    let alive: Vec<&PubkeyRecord> = contents
        .records
        .iter()
        .filter(|record| is_process_running(record.pid))
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
                .is_none_or(|record| is_process_running(record.pid))
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
        file.sync_all()?;
        if fs::read(&db_path)? != before {
            return Err(io::Error::other(
                "store changed during cleanup; stopped without replacing it",
            ));
        }
        fs::rename(&temporary, &db_path)
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

fn get(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = db_path()?;

    let contents =
        storage::read(&db_path, MAX_STORE)?.ok_or("database does not exist; run init or add")?;
    reject_malformed(&contents)?;

    if let Some(record) = latest_for_pid(&contents.records, pid) {
        println!("{}", record.pubkey);
    } else {
        return Err(format!("no public key found for PID {pid}").into());
    }

    Ok(())
}

fn db_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let xdg_state =
        std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{home}/.local/state"));
    Ok(PathBuf::from(xdg_state).join("kitty"))
}

fn db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(db_dir()?.join("pubkey.tsv"))
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

const MAX_STORE: usize = 16 * 1024 * 1024;

fn reject_malformed(
    contents: &kitty_rc_key_store::StoreContents,
) -> Result<(), Box<dyn std::error::Error>> {
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
