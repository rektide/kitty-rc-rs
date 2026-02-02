use clap::{Parser, Subcommand};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
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
    /// Add a public key entry
    Add {
        /// PID of the kitty instance
        pid: u32,
        /// Base85 encoded public key
        pubkey: String,
        /// Window ID (optional)
        #[arg(long)]
        window_id: Option<u32>,
    },
    /// Clean up stale entries
    Cleanup,
    /// Get public key for a PID
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
    let db_dir = get_db_dir()?;
    fs::create_dir_all(&db_dir)?;

    let db_path = get_db_path()?;
    if !db_path.exists() {
        File::create(&db_path)?;
        println!("Created database: {}", db_path.display());
    } else {
        println!("Database already exists: {}", db_path.display());
    }

    let epoch_path = get_epoch_path()?;
    if !epoch_path.exists() {
        write_current_time(&epoch_path)?;
        println!("Created epoch file: {}", epoch_path.display());
    }

    print!("\nAdd this to your ~/.zshrc:\n");
    print!("  # Record kitty public key when shell starts\n");
    print!("  if [[ -n \"$KITTY_PUBLIC_KEY\" && -n \"$KITTY_PID\" ]]; then\n");
    print!(
        r#"      kitty-pubkey-db add "$KITTY_PID" "${{KITTY_WINDOW_ID-}}" "$KITTY_PUBLIC_KEY" &"#
    );
    print!("\n");
    print!("      disown\n");
    print!("  fi\n");

    Ok(())
}

fn add(pid: u32, window_id: Option<u32>, pubkey: String) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path()?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let window_id_str = window_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "".to_string());

    let entry = format!("{}\t{}\t{}\t{}", pid, window_id_str, pubkey, timestamp);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&db_path)?;

    writeln!(file, "{}", entry)?;
    file.flush()?;

    check_and_cleanup_if_needed(&db_path)?;

    Ok(())
}

fn cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path()?;
    let epoch_path = get_epoch_path()?;

    let _current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    write_current_time(&epoch_path)?;

    if !db_path.exists() {
        println!("Database does not exist");
        return Ok(());
    }

    let content = fs::read_to_string(&db_path)?;
    let entries: Vec<DbEntry> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| parse_db_entry(line))
        .collect();

    let alive_entries: Vec<DbEntry> = entries
        .iter()
        .filter(|entry| is_process_running(entry.pid))
        .cloned()
        .collect();

    let mut file = File::create(&db_path)?;
    for entry in &alive_entries {
        writeln!(
            file,
            "{}\t{}\t{}\t{}",
            entry.pid, entry.window_id, entry.pubkey, entry.timestamp
        )?;
    }
    file.flush()?;

    println!(
        "Cleanup complete: kept {} of {} entries",
        alive_entries.len(),
        entries.len()
    );

    Ok(())
}

fn get(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path()?;

    if !db_path.exists() {
        eprintln!("Database does not exist");
        return Ok(());
    }

    let content = fs::read_to_string(&db_path)?;
    let entries: Vec<DbEntry> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| parse_db_entry(line))
        .filter(|entry| entry.pid == pid)
        .collect();

    if let Some(entry) = entries.into_iter().max_by_key(|e| e.timestamp) {
        println!("{}", entry.pubkey);
    } else {
        eprintln!("No public key found for PID {}", pid);
        std::process::exit(1);
    }

    Ok(())
}

fn get_db_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let xdg_state = env::var("XDG_STATE_HOME").unwrap_or(format!("{}/.local/state", home));

    let db_dir = PathBuf::from(xdg_state).join("kitty");
    Ok(db_dir)
}

fn get_db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(get_db_dir()?.join("pubkey.tsv"))
}

fn get_epoch_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(get_db_dir()?.join("pubkey-check.epoch"))
}

fn write_current_time(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    fs::write(path, format!("{}", current_time))?;
    Ok(())
}

fn check_and_cleanup_if_needed(_db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let epoch_path = get_epoch_path()?;

    if !epoch_path.exists() {
        write_current_time(&epoch_path)?;
        return Ok(());
    }

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let last_cleanup = fs::read_to_string(&epoch_path)?.trim().parse::<u64>()?;

    if current_time - last_cleanup >= 86400 {
        cleanup()?;
    }

    Ok(())
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Path::new(&format!("/proc/{}", pid)).exists()
    }

    #[cfg(not(unix))]
    {
        true
    }
}

#[derive(Debug, Clone)]
struct DbEntry {
    pid: u32,
    window_id: String,
    pubkey: String,
    timestamp: u64,
}

fn parse_db_entry(line: &str) -> Option<DbEntry> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 3 {
        return None;
    }

    let pid = parts[0].parse::<u32>().ok()?;
    let window_id = parts[1].to_string();
    let pubkey = parts[2].to_string();
    let timestamp = parts
        .get(3)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    Some(DbEntry {
        pid,
        window_id,
        pubkey,
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_db_entry() {
        let line = "12345\t67890\t1:abc123\t1704067200";
        let entry = parse_db_entry(line).unwrap();
        assert_eq!(entry.pid, 12345);
        assert_eq!(entry.window_id, "67890");
        assert_eq!(entry.pubkey, "1:abc123");
        assert_eq!(entry.timestamp, 1704067200);
    }

    #[test]
    fn test_parse_db_entry_no_window_id() {
        let line = "12345\t\t1:abc123\t1704067200";
        let entry = parse_db_entry(line).unwrap();
        assert_eq!(entry.pid, 12345);
        assert_eq!(entry.window_id, "");
        assert_eq!(entry.pubkey, "1:abc123");
        assert_eq!(entry.timestamp, 1704067200);
    }

    #[test]
    fn test_parse_db_entry_no_timestamp() {
        let line = "12345\t67890\t1:abc123";
        let entry = parse_db_entry(line).unwrap();
        assert_eq!(entry.pid, 12345);
        assert_eq!(entry.window_id, "67890");
        assert_eq!(entry.pubkey, "1:abc123");
        assert_eq!(entry.timestamp, 0);
    }
}
