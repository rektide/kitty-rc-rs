use kitty_rc::{Kitty};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find kitty socket
    let socket_path = find_kitty_socket()?;

    println!("Connecting to: {}", socket_path);

    // Test encrypted connection
    let mut kitty = Kitty::builder()
        .socket_path(&socket_path)
        .password(&read_password()?)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    println!("✓ Connected successfully (encrypted)!");

    // Test sending a simple command
    use kitty_rc::command::CommandBuilder;

    let cmd = CommandBuilder::new("ls")
        .build();

    match kitty.execute(&cmd).await {
        Ok(response) => {
            println!("✓ Encrypted command executed successfully!");
            println!("  Response: {:?}", response);
        }
        Err(e) => {
            eprintln!("✗ Encrypted command failed: {}", e);
            if let Some(source) = Error::source(&e) {
                eprintln!("  Error source: {}", source);
            }
        }
    }

    Ok(())
}

fn read_password() -> Result<String, Box<dyn std::error::Error>> {
    let password_file = format!("{}/.config/kitty/rc.password",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
    std::fs::read_to_string(&password_file)
        .map_err(|e| Box::<dyn std::error::Error>::from(e))
        .map(|s| s.to_string())
}

fn find_kitty_socket() -> Result<String, Box<dyn std::error::Error>> {
    // Try XDG runtime directory first
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::Path::new(&runtime).join("kitty");
        if let Some(sock) = find_socket_in_dir(&dir) {
            return Ok(sock);
        }
    }

    // Try /run/user/<uid>/kitty
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let dir = format!("/run/user/{}/kitty", uid);
    let dir = std::path::Path::new(&dir);
    if let Some(sock) = find_socket_in_dir(&dir) {
        return Ok(sock);
    }

    // Try /tmp/kitty
    let dir = std::path::Path::new("/tmp/kitty");
    if let Some(sock) = find_socket_in_dir(&dir) {
        return Ok(sock);
    }

    Err("Could not find kitty socket. Please ensure kitty is running with remote control enabled.".into())
}

fn find_socket_in_dir(dir: &std::path::Path) -> Option<String> {
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".sock") {
                    return Some(dir.join(name).to_string_lossy().to_string());
                }
            }
        }
    }
    None
}
