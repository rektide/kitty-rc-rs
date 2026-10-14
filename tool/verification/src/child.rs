//! `kitty-rc-verify-child` — the process private kitty fixtures run inside
//! their terminal window. Launched by kitty (never by the runner directly),
//! it publishes the fixture key, provides sentinel/stream/sleep journeys and
//! replaces the former Python `-c` children. See `src/childbin.rs` for the
//! runner side of this contract.

use clap::{Parser, Subcommand};
use std::io::{Read, Write};

#[derive(Parser)]
#[command(
    name = "kitty-rc-verify-child",
    about = "Fixture child for private kitty verification windows",
    long_about = "Launched by kitty inside an isolated verification fixture. \
                  Publishes KITTY_PUBLIC_KEY, emits startup markers, and provides \
                  sentinel, raw-stream and sleep journeys for the examples."
)]
struct ChildCli {
    #[command(subcommand)]
    command: ChildCommand,
}

#[derive(Subcommand)]
enum ChildCommand {
    /// Write $KITTY_PUBLIC_KEY to a file, print a marker, then sleep.
    Key {
        /// Path that receives the fixture's published public key.
        key_file: String,
        /// Text printed to the terminal; observed later via get-text.
        #[arg(long, default_value = "kitty-rc private fixture")]
        marker: String,
        /// How long to stay alive before exiting.
        #[arg(long, default_value_t = 120)]
        sleep_secs: u64,
    },
    /// Write "started" to a path, sleep, then exit with a chosen code.
    Sentinel {
        /// Path that receives the "started" marker.
        path: String,
        #[arg(long, default_value_t = 1500)]
        sleep_ms: u64,
        #[arg(long, default_value_t = 7)]
        exit: i32,
    },
    /// Copy stdin to stdout byte-for-byte, emit fixed stderr bytes, exit.
    Stream {
        /// Hex bytes written to stderr before exiting.
        #[arg(long, default_value = "")]
        stderr_hex: String,
        #[arg(long, default_value_t = 0)]
        exit: i32,
    },
    /// Sleep without any other effect.
    Sleep {
        #[arg(long, default_value_t = 200)]
        ms: u64,
    },
}

fn main() {
    let cli = ChildCli::parse();
    let code = match cli.command {
        ChildCommand::Key {
            key_file,
            marker,
            sleep_secs,
        } => {
            let key = std::env::var("KITTY_PUBLIC_KEY").unwrap_or_else(|error| {
                eprintln!("kitty-rc-verify-child: KITTY_PUBLIC_KEY missing: {error}");
                std::process::exit(2);
            });
            if let Err(error) = std::fs::write(&key_file, key) {
                eprintln!("kitty-rc-verify-child: cannot publish key: {error}");
                std::process::exit(2);
            }
            println!("{marker}");
            let _ = std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_secs(sleep_secs));
            0
        }
        ChildCommand::Sentinel {
            path,
            sleep_ms,
            exit,
        } => {
            if let Err(error) = std::fs::write(&path, "started") {
                eprintln!("kitty-rc-verify-child: cannot write sentinel: {error}");
                std::process::exit(2);
            }
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
            exit
        }
        ChildCommand::Stream { stderr_hex, exit } => {
            let stderr_bytes = decode_hex(&stderr_hex).unwrap_or_else(|error| {
                eprintln!("kitty-rc-verify-child: invalid --stderr-hex: {error}");
                std::process::exit(2);
            });
            let mut stdin = std::io::stdin().lock();
            let mut stdout = std::io::stdout().lock();
            let mut buffer = [0u8; 65536];
            loop {
                match stdin.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => {
                        if stdout.write_all(&buffer[..read]).is_err() {
                            break;
                        }
                        let _ = stdout.flush();
                    }
                }
            }
            let _ = std::io::stderr().write_all(&stderr_bytes);
            exit
        }
        ChildCommand::Sleep { ms } => {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            0
        }
    };
    std::process::exit(code);
}

/// Decode an even-length hex string (empty allowed).
fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if !text.len().is_multiple_of(2) || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected even-length hex".into());
    }
    Ok(text
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap() as u8;
            let low = (pair[1] as char).to_digit(16).unwrap() as u8;
            (high << 4) | low
        })
        .collect())
}
