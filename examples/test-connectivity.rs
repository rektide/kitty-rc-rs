use kitty_rc::Kitty;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find kitty socket
    let socket_path = find_kitty_socket()?;

    println!("Connecting to: {}", socket_path);

    // Test basic connection
    let mut kitty = Kitty::builder()
        .socket_path(&socket_path)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    println!("✓ Connected successfully!");

    // Test sending a simple command
    use kitty_rc::command::CommandBuilder;

    let cmd = CommandBuilder::new("ls")
        .build();

    match kitty.execute(&cmd).await {
        Ok(response) => {
            println!("✓ Command executed successfully!");
            if let Some(data) = response.data {
                println!("Response: {}", data);
            }
        }
        Err(e) => {
            eprintln!("✗ Command failed: {}", e);
        }
    }

    Ok(())
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
