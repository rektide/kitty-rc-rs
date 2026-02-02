# Plan: Kitty Public Key Database

## Problem

Kitty generates its own X25519 key pair for password authentication and exposes its public key via the `KITTY_PUBLIC_KEY` environment variable. However, this environment variable is only set for **processes launched by kitty**. Standalone clients cannot access kitty's public key because:
1. Kitty does not expose its public key via any socket command
2. Kitty does not write its public key to any file
3. There is no documented way to retrieve it

## Solution

Create a small shell hook program that runs when a shell is started by kitty. This program:
1. Records the public key and environment identifiers
2. Stores them in a persistent database
3. Periodically cleans up stale entries

The database can then be read by standalone clients to find the appropriate public key for a kitty instance.

## Design

### Database Location

Store the database in kitty's XDG data directory:
- `~/.local/state/kitty/pubkey.tsv` (or appropriate XDG location)

### Database Format

Tab-separated values (TSV) with one entry per shell:

```
PID\tWINDOW_ID\tPUBLIC_KEY\tTIMESTAMP
```

Fields:
- **PID**: The `KITTY_PID` of the kitty instance
- **WINDOW_ID**: The `KITTY_WINDOW_ID` of the shell window
- **PUBLIC_KEY**: The `KITTY_PUBLIC_KEY` (Base85 encoded)
- **TIMESTAMP**: Unix timestamp when this entry was created

### Cleanup Tracking File

Store a file `pubkey-check.epoch` containing:
- Unix timestamp of last cleanup run

### Cleanup Logic

When the hook runs:
1. Check `pubkey-check.epoch`
2. If older than 1 day (86400 seconds):
   - Update `pubkey-check.epoch` to current time
   - For each entry in `pubkey.tsv`:
     - Check if a process with that PID is running
     - If not, remove the entry
3. If `pubkey-check.epoch` doesn't exist:
   - Create it with current timestamp
   - Skip cleanup on first run

## Implementation

### Shell Hook (for `.zshrc`)

```bash
# Kitty public key database hook
if [[ -n "$KITTY_PUBLIC_KEY" && -n "$KITTY_PID" ]]; then
    kitty-pubkey-db add "$KITTY_PID" "$KITTY_WINDOW_ID" "$KITTY_PUBLIC_KEY" &
fi
```

### Program: `kitty-pubkey-db`

A small Rust program with subcommands:

#### `add` subcommand

```rust
pub fn add(pid: u32, window_id: Option<u32>, pubkey: String) -> Result<()> {
    let db_path = get_db_path()?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Append entry to TSV file
    let entry = format!(
        "{}\t{}\t{}\t{}",
        pid,
        window_id.map(|id| id.to_string()).unwrap_or_else(|| "".to_string()),
        pubkey,
        timestamp
    );

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&db_path)?;

    writeln!(file, "{}", entry)?;

    // Periodically check if cleanup is needed
    check_and_cleanup_if_needed(&db_path)?;

    Ok(())
}
```

#### `cleanup` subcommand

```rust
pub fn cleanup() -> Result<()> {
    let db_path = get_db_path()?;
    let epoch_path = get_epoch_path()?;

    // Read current timestamp
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Update epoch file
    write(&epoch_path, format!("{}", current_time))?;

    // Read existing entries
    let entries: Vec<DbEntry> = read_db_entries(&db_path)?;

    // Filter to keep only entries with running PIDs
    let alive_entries: Vec<DbEntry> = entries
        .into_iter()
        .filter(|entry| is_process_running(entry.pid))
        .collect();

    // Rewrite database with only alive entries
    rewrite_database(&db_path, &alive_entries)?;

    println!(
        "Cleanup complete: kept {} of {} entries",
        alive_entries.len(),
        entries.len()
    );

    Ok(())
}

fn is_process_running(pid: u32) -> bool {
    // Check if /proc/<pid> exists (Linux) or use appropriate method
    Path::new(format!("/proc/{}", pid)).exists()
}
```

#### `get` subcommand (for clients)

```rust
pub fn get(pid: u32) -> Result<Option<String>> {
    let db_path = get_db_path()?;
    let entries: Vec<DbEntry> = read_db_entries(&db_path)?;

    // Find most recent entry with matching PID
    let entry = entries
        .into_iter()
        .filter(|e| e.pid == pid)
        .max_by_key(|e| e.timestamp);

    Ok(entry.map(|e| e.public_key))
}
```

### Database Helper Functions

```rust
fn get_db_path() -> Result<PathBuf> {
    let xdg_state = env::var("XDG_STATE_HOME")
        .or_else(|| env::var("HOME").map(|h| format!("{}/.local/state", h)))?;

    let db_dir = PathBuf::from(xdg_state).join("kitty");
    fs::create_dir_all(&db_dir)?;

    Ok(db_dir.join("pubkey.tsv"))
}

fn get_epoch_path() -> Result<PathBuf> {
    let db_dir = get_db_path()?.parent().unwrap();
    Ok(db_dir.join("pubkey-check.epoch"))
}

struct DbEntry {
    pid: u32,
    window_id: u32,
    public_key: String,
    timestamp: u64,
}

fn read_db_entries(path: &Path) -> Result<Vec<DbEntry>> {
    let content = fs::read_to_string(path)?;
    content
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| parse_db_entry(line))
        .collect()
}

fn parse_db_entry(line: &str) -> Result<DbEntry> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 3 {
        return Err(anyhow!("Invalid entry format"));
    }

    let pid: u32 = parts[0].parse()?;
    let window_id: u32 = parts[1].parse()?;
    let public_key = parts[2].to_string();
    let timestamp: u64 = parts[3].unwrap_or(&"0").parse()?;

    Ok(DbEntry { pid, window_id, public_key, timestamp })
}

fn check_and_cleanup_if_needed(db_path: &Path) -> Result<()> {
    let epoch_path = get_epoch_path()?;
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Check if cleanup is needed
    if epoch_path.exists() {
        let last_cleanup = fs::read_to_string(&epoch_path)?.parse::<u64>()?;
        if current_time - last_cleanup < 86400 {
            // Less than 1 day old, skip cleanup
            return Ok(());
        }
    }

    // Run cleanup
    cleanup()
}
```

## Usage Examples

### For End Users

Add to `~/.zshrc`:

```bash
# Source kitty-pubkey-db if available
if command -v kitty-pubkey-db &>/dev/null; then
    kitty-pubkey-db init &>/dev/null 2>&1
fi

# Hook to record public key when shell is started by kitty
if [[ -n "$KITTY_PUBLIC_KEY" && -n "$KITTY_PID" ]]; then
    kitty-pubkey-db add "$KITTY_PID" "${KITTY_WINDOW_ID-}" "$KITTY_PUBLIC_KEY" &
    disown
fi
```

### For Application Developers

```rust
use std::env;

// Find kitty PID (e.g., by finding socket file)
let kitty_pid = find_kitty_pid()?;

// Get public key from database
let pubkey = match kitty_pubkey_db::get(kitty_pid) {
    Ok(Some(key)) => key,
    Ok(None) => {
        eprintln!("No public key found for PID {}", kitty_pid);
        // Fallback or prompt user
        return Err(KittyError::MissingPublicKey);
    }
    Err(e) => {
        eprintln!("Error reading pubkey db: {}", e);
        return Err(KittyError::DatabaseError(e));
    }
};

// Use public key for encrypted communication
let encryptor = Encryptor::from_public_key(&pubkey)?;
```

## Benefits

1. **Works with standalone clients**: No need to run within kitty
2. **Automatic**: Records keys as shells are created
3. **Self-cleaning**: Removes stale entries periodically
4. **Simple**: TSV format is easy to debug and process
5. **Efficient**: Cleanup only runs once per day, not on every shell start

## Alternatives Considered

### 1. Kitty exposes public key via socket command
**Pros**: Cleaner, more direct
**Cons**: Requires kitty source code changes

### 2. Write public key to file on startup
**Pros**: Simple for clients to read
**Cons**: Not implemented in kitty, would require patching

### 3. Store in shared memory
**Pros**: Fast, no disk I/O
**Cons**: Complex, security concerns

The database approach was chosen because:
- Requires no kitty changes
- Works with existing kitty versions
- Minimal performance impact
- Easy to implement and debug
