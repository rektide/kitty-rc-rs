# kitty-rc

> A Rust library for controlling the kitty terminal emulator via its remote control protocol

## About

kitty-rc is a Rust library that provides a type-safe, async interface for controlling the kitty terminal emulator through its remote control protocol. It uses Unix domain sockets for communication and supports all major kitty remote control commands.

## Features

- **Comprehensive Command Support**: All major kitty remote control commands organized into modules
- **Type-Safe Builder API**: Fluent builder pattern for client setup and all commands
- **Async-First**: Built on tokio for asynchronous I/O
- **Single Connection**: Connect once at startup, reuse for all commands
- **Error Handling**: Detailed error types for protocol, command, and connection errors
- **Streaming Support**: Automatic chunking for large payloads (e.g., background images)
- **Async Commands**: Support for async operations with unique ID generation
- **Comprehensive Testing**: 143 unit tests ensuring reliability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kitty-rc = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Usage

### Basic Setup

```rust
use kitty_rc::Kitty;

#[tokio::main]
async fn main() -> Result<(), kitty_rc::KittyError> {
    let mut kitty = Kitty::builder()
        .socket_path("/path/to/kitty.socket")
        .connect()
        .await?;

    // Use kitty to send commands...
    Ok(())
}
```

### Listing Windows

```rust
use kitty_rc::{LsCommand, WindowInfo};

let cmd = LsCommand::new()
    .self_window(true)
    .build()?;

let response = kitty.execute(&cmd).await?;
let instances = LsCommand::parse_response(&response)?;

for instance in &instances {
    for tab in &instance.tabs {
        for window in &tab.windows {
            println!("Window: {} (id: {:?})", window.title, window.id);
            println!("  PID: {:?}", window.pid);
            println!("  CWD: {:?}", window.cwd);
        }
    }
}
```

### Sending Text to Windows

```rust
use kitty_rc::commands::SendTextCommand;

let cmd = SendTextCommand::new("text:hello world")
    .match_spec("id:1")
    .build()?;

kitty.send_command(&cmd).await?;
```

### Creating New Windows

```rust
use kitty_rc::commands::NewWindowCommand;

let cmd = NewWindowCommand::new()
    .args("bash")
    .cwd("/home/user")
    .title("My Window")
    .build()?;

kitty.send_command(&cmd).await?;
```

### Setting Font Size

```rust
use kitty_rc::commands::SetFontSizeCommand;

// Set absolute font size
let cmd = SetFontSizeCommand::new(16)
    .all(true)
    .build()?;

kitty.execute(&cmd).await?;

// Increment font size
let cmd = SetFontSizeCommand::new(0)
    .increment_op("+")
    .build()?;

kitty.execute(&cmd).await?;

// Decrement font size
let cmd = SetFontSizeCommand::new(0)
    .increment_op("-")
    .build()?;

kitty.execute(&cmd).await?;
```

### Setting Background Opacity

```rust
use kitty_rc::commands::SetBackgroundOpacityCommand;

let cmd = SetBackgroundOpacityCommand::new(0.8)
    .all(true)
    .build()?;

kitty.send_command(&cmd).await?;
```

### Streaming Large Data

```rust
use kitty_rc::commands::SetBackgroundImageCommand;

let large_image_data = "..."; // Large base64 string
let cmd = SetBackgroundImageCommand::new(large_image_data).build()?;

// send_all automatically handles chunking for large payloads
kitty.send_all(&cmd.unwrap()).await?;
```

Or to get a response:

```rust
let response = kitty.execute_all(&cmd.unwrap()).await?;
```

## Command Modules

### Tab Management (`commands::tab`)
- `CloseTabCommand` - Close tabs
- `DetachTabCommand` - Detach tabs to different OS windows
- `FocusTabCommand` - Focus a specific tab
- `SetTabTitleCommand` - Set tab title

### Layout Management (`commands::layout`)
- `GotoLayoutCommand` - Switch to a specific layout
- `SetEnabledLayoutsCommand` - Set available layouts
- `LastUsedLayoutCommand` - Switch to last used layout

### Window Management (`commands::window`)
- `CloseWindowCommand` - Close windows
- `CreateMarkerCommand` - Create scroll markers
- `DetachWindowCommand` - Detach windows
- `FocusWindowCommand` - Focus a window
- `GetTextCommand` - Extract text from windows
- `LsCommand` - List windows and tabs (supports structured response parsing)
- `NewWindowCommand` - Create new windows
- `RemoveMarkerCommand` - Remove scroll markers
- `ResizeWindowCommand` - Resize windows
- `ScrollWindowCommand` - Scroll window content
- `SelectWindowCommand` - Async window selection
- `SendKeyCommand` - Send keyboard shortcuts
- `SendTextCommand` - Send text to windows
- `SetWindowLogoCommand` - Set window logo
- `SetWindowTitleCommand` - Set window title

### Process Management (`commands::process`)
- `DisableLigaturesCommand` - Disable font ligatures
- `EnvCommand` - Set environment variables
- `KittenCommand` - Run kitty kittens
- `LaunchCommand` - Launch new windows with comprehensive options
- `LoadConfigCommand` - Load configuration files
- `ResizeOSWindowCommand` - Resize OS windows
- `RunCommand` - Run commands with streaming support
- `SetUserVarsCommand` - Set user variables
- `SignalChildCommand` - Send signals to child processes

### Style and Appearance (`commands::style`)
- `GetColorsCommand` - Get current colors
- `SetBackgroundImageCommand` - Set background image (supports streaming)
- `SetBackgroundOpacityCommand` - Set background opacity
- `SetColorsCommand` - Set color scheme
- `SetFontSizeCommand` - Set font size (absolute or increment/decrement)
- `SetSpacingCommand` - Set window padding/spacing
- `SetTabColorCommand` - Set tab colors

> **Note**: Kitty's remote protocol does not include a `get-font-size` command. To retrieve the current font size, read it from `~/.config/kitty/kitty.conf`.

## Response Types

The library provides structured types for parsing kitty responses:

### Window Information
```rust
use kitty_rc::{WindowInfo, LsCommand};

let response = kitty.execute(&cmd).await?;
let instances = LsCommand::parse_response(&response)?;

for instance in &instances {
    for tab in &instance.tabs {
        for window in &tab.windows {
            println!("Window: {} (id: {:?})", window.title, window.id);
            println!("  PID: {:?}", window.pid);
            println!("  CWD: {:?}", window.cwd);
            println!("  Active: {:?}", window.is_active);
            println!("  Focused: {:?}", window.is_focused);
        }
    }
}
```

The `WindowInfo` struct includes all fields returned by kitty:
- Window metadata: `id`, `title`, `pid`, `cwd`, `cmdline`
- State: `is_active`, `is_focused`, `is_self`, `at_prompt`, `in_alternate_screen`
- Terminal: `columns`, `lines`, `created_at`
- Processes: `foreground_processes`, `env`, `user_vars`
- Command: `last_cmd_exit_status`, `last_reported_cmdline`

The `TabInfo` struct includes tab-level information:
- Tab metadata: `id`, `title`
- State: `is_active`, `is_focused`, `active_window_history`
- Layout: `layout`, `enabled_layouts`, `layout_opts`, `layout_state`
- Groups: `groups` (window grouping information)

The `OsInstance` struct includes OS-level information:
- OS metadata: `id`, `wm_class`, `wm_name`
- State: `is_active`, `is_focused`, `last_focused`
- Display: `background_opacity`, `platform_window_id`

## Async and Streaming

The library supports async commands and streaming for large payloads:

### Async Commands
```rust
use kitty_rc::commands::SelectWindowCommand;
use kitty_rc::protocol::KittyMessage;

let async_id = KittyMessage::generate_unique_id();

let cmd = SelectWindowCommand::new()
    .title("Select a window")
    .async_id(async_id.clone())
    .build()?;
```

### Streaming
Large payloads (>4096 bytes) can be sent using the helper methods that automatically handle chunking:

```rust
let message = cmd.build()?;
client.send_all(&message).await?;  // Automatically chunks if needed
```

Or to get a response:

```rust
let response = client.execute_all(&message).await?;
```

The `send_all` and `execute_all` methods automatically detect if a message needs streaming and handle chunking internally, so you don't need to manually check or split payloads.

## Error Handling

kitty-rc provides detailed error types:

```rust
use kitty_rc::{KittyError, CommandError};

match result {
    Ok(response) => println!("Success: {:?}", response),
    Err(KittyError::Command(CommandError::MissingParameter(field, _))) => {
        eprintln!("Missing required parameter: {}", field);
    }
    Err(KittyError::Connection(err)) => {
        eprintln!("Connection error: {}", err);
    }
    Err(err) => eprintln!("Error: {}", err),
}
```

## Enabling Remote Control

To use kitty-rc, you must enable remote control in kitty. The library provides a helper script to configure kitty securely with password authentication.

### Quick Setup

Run the included setup script:

```bash
./scripts/enable-rc.sh
```

This script will:
- Generate a random 48-character password in `~/.config/kitty/rc.password`
- Create `~/.config/kitty/rc.conf` with remote control configuration
- Add `include rc.conf` to `~/.config/kitty/kitty.conf` if not present
- Set up a socket in the XDG runtime directory at `kitty/kitty-{kitty_pid}.sock`

### Manual Configuration

To manually configure kitty for remote control, create `~/.config/kitty/rc.conf`:

```conf
# Enable password-based remote control
allow_remote_control password
remote_control_password "$(cat ~/.config/kitty/rc.password)"

# Listen on socket in XDG runtime directory
listen_on unix:${XDG_RUNTIME_DIR}/kitty/kitty-{kitty_pid}.sock
```

Then add to `~/.config/kitty/kitty.conf`:

```conf
include rc.conf
```

This script will:
- Generate a random 48-character password in `~/.config/kitty/rc.password`
- Add remote control configuration to `~/.config/kitty/kitty.conf`
- Set up a socket in the XDG runtime directory at `kitty/kitty-{pid}.sock`
- Fail if any remote control settings are already configured

### Manual Configuration

To manually configure kitty for remote control, add to `~/.config/kitty/kitty.conf`:

```conf
# Enable password-based remote control
allow_remote_control password
remote_control_password "$(cat ~/.config/kitty/rc.password)"

# Listen on socket in XDG runtime directory
listen_on unix:${XDG_RUNTIME_DIR}/kitty/kitty-{kitty_pid}.sock
```

Generate a secure password:

```bash
pwgen -s 48 1 > ~/.config/kitty/rc.password
chmod 600 ~/.config/kitty/rc.password
```

Restart kitty after making changes.

## Socket Configuration

kitty-rc communicates with kitty via Unix domain sockets. The default socket location varies by system:

- With the helper script: `$XDG_RUNTIME_DIR/kitty/kitty-{kitty_pid}.sock`
- Linux/macOS: `$TMPDIR/kitty-*` or `/tmp/kitty-*`
- You can also specify custom socket paths

## Testing

Run the test suite:

```bash
cargo test
```

All 143 tests pass successfully.

## Examples

See `src/bin/list_windows.rs` for a complete example of listing kitty windows and their processes.

Use `scripts/enable-rc.sh` to quickly enable remote control in your kitty configuration.

## Contributing

Contributions are welcome! Please read the project's AGENTS.md for development guidelines.

## License

[Specify your license here]

## Related

- [kitty terminal emulator](https://sw.kovidgoyal.net/kitty/)
- [kitty remote control protocol documentation](https://sw.kovidgoyal.net/kitty/remote-control/)

## Acknowledgments

Built with:
- [tokio](https://tokio.rs/) - Async runtime
- [serde](https://serde.rs/) - Serialization framework
- [thiserror](https://docs.rs/thiserror/) - Error derive macros
