//! `kitty-pubkey-db`: manage the persisted store of kitty server public keys
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
use kitty_rc_key_store::{PubkeyRecord, latest_for_pid, parse_store};
use std::fs::{self, File, OpenOptions};
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
    Cleanup,
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
        Commands::Cleanup => cleanup()?,
        Commands::Get { pid } => get(pid)?,
    }

    Ok(())
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    let db_dir = db_dir()?;
    fs::create_dir_all(&db_dir)?;

    let db_path = db_path()?;
    if !db_path.exists() {
        File::create(&db_path)?;
        println!("Created database: {}", db_path.display());
    } else {
        println!("Database already exists: {}", db_path.display());
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

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(db_path()?)?;
    writeln!(file, "{}", record.render())?;
    file.flush()?;

    Ok(())
}

fn cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = db_path()?;

    if !db_path.exists() {
        println!("Database does not exist");
        return Ok(());
    }

    let contents = parse_store(&fs::read_to_string(&db_path)?);
    for (line, error) in &contents.malformed {
        eprintln!("malformed record at line {line}: {error}");
    }

    let alive: Vec<&PubkeyRecord> = contents
        .records
        .iter()
        .filter(|record| is_process_running(record.pid))
        .collect();
    let removed = contents.records.len() - alive.len();

    let mut file = File::create(&db_path)?;
    for record in &alive {
        writeln!(file, "{}", record.render())?;
    }
    file.flush()?;

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

    if !db_path.exists() {
        eprintln!("Database does not exist");
        std::process::exit(1);
    }

    let contents = parse_store(&fs::read_to_string(&db_path)?);
    for (line, error) in &contents.malformed {
        eprintln!("malformed record at line {line}: {error}");
    }

    if let Some(record) = latest_for_pid(&contents.records, pid) {
        println!("{}", record.pubkey);
    } else {
        eprintln!("No public key found for PID {pid}");
        std::process::exit(1);
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
