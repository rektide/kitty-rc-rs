# kitty-rc

> A Rust library for controlling the kitty terminal emulator via its remote control protocol

## About

kitty-rc is a Rust library that provides a type-safe, async interface for controlling the kitty terminal emulator through its remote control protocol. It uses Unix domain sockets for communication and supports all major kitty remote control commands.

## Features

- **Comprehensive Command Support**: All major kitty remote control commands organized into modules
- **Type-Safe Builder API**: Fluent builder pattern for all commands
- **Async-First**: Built on tokio for asynchronous I/O
- **Connection Pooling**: Reusable connections for better performance
- **Error Handling**: Detailed error types for protocol, command, and connection errors
- **Streaming Support**: Automatic chunking for large payloads (e.g., background images)
- **Async Commands**: Support for async operations with unique ID generation
- **Comprehensive Testing**: 123 unit tests ensuring reliability

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
use kitty_rc::KittyClient;

#[tokio::main]
async fn main() -> Result<(), kitty_rc::KittyError> {
    let client = KittyClient::new("/path/to/kitty.socket").await?;
    
    // Use client to send commands...
    Ok(())
}
```

### Listing Windows

```rust
use kitty_rc::commands::LsCommand;

let cmd = LsCommand::new()
    .self_window(true)
    .build()?;

let response = client.send_command(&cmd).await?;
println!("{:?}", response);
```

### Sending Text to Windows

```rust
use kitty_rc::commands::SendTextCommand;

let cmd = SendTextCommand::new("text:hello world")
    .match_spec("id:1")
    .build()?;

client.send_command(&cmd).await?;
```

### Creating New Windows

```rust
use kitty_rc::commands::NewWindowCommand;

let cmd = NewWindowCommand::new()
    .args("bash")
    .cwd("/home/user")
    .title("My Window")
    .build()?;

client.send_command(&cmd).await?;
```

### Setting Background Opacity

```rust
use kitty_rc::commands::SetBackgroundOpacityCommand;

let cmd = SetBackgroundOpacityCommand::new(0.8)
    .all(true)
    .build()?;

client.send_command(&cmd).await?;
```

### Streaming Large Data

```rust
use kitty_rc::commands::SetBackgroundImageCommand;
use kitty_rc::protocol::KittyMessage;

let large_image_data = "..."; // Large base64 string
let cmd = SetBackgroundImageCommand::new(large_image_data).build()?;

let message = cmd.unwrap();
let chunks = message.into_chunks();

for chunk in chunks {
    client.send_raw(&chunk).await?;
}
```

## Command Modules

### Tab Management (`commands::tab`)
- `FocusTabCommand` - Focus a specific tab
- `SetTabTitleCommand` - Set tab title
- `CloseTabCommand` - Close tabs
- `DetachTabCommand` - Detach tabs to different OS windows

### Layout Management (`commands::layout`)
- `GotoLayoutCommand` - Switch to a specific layout
- `SetEnabledLayoutsCommand` - Set available layouts
- `LastUsedLayoutCommand` - Switch to last used layout

### Window Management (`commands::window`)
- `LsCommand` - List windows and tabs
- `SendTextCommand` - Send text to windows
- `SendKeyCommand` - Send keyboard shortcuts
- `CloseWindowCommand` - Close windows
- `ResizeWindowCommand` - Resize windows
- `FocusWindowCommand` - Focus a window
- `SelectWindowCommand` - Async window selection
- `NewWindowCommand` - Create new windows
- `DetachWindowCommand` - Detach windows
- `SetWindowTitleCommand` - Set window title
- `SetWindowLogoCommand` - Set window logo
- `GetTextCommand` - Extract text from windows
- `ScrollWindowCommand` - Scroll window content
- `CreateMarkerCommand` - Create scroll markers
- `RemoveMarkerCommand` - Remove scroll markers

### Process Management (`commands::process`)
- `RunCommand` - Run commands with streaming support
- `KittenCommand` - Run kitty kittens
- `LaunchCommand` - Launch new windows with comprehensive options
- `EnvCommand` - Set environment variables
- `SetUserVarsCommand` - Set user variables
- `LoadConfigCommand` - Load configuration files
- `ResizeOSWindowCommand` - Resize OS windows
- `DisableLigaturesCommand` - Disable font ligatures
- `SignalChildCommand` - Send signals to child processes

### Style and Appearance (`commands::style`)
- `SetBackgroundOpacityCommand` - Set background opacity
- `SetBackgroundImageCommand` - Set background image (supports streaming)
- `SetColorsCommand` - Set color scheme
- `SetFontSizeCommand` - Set font size
- `SetSpacingCommand` - Set window padding/spacing
- `SetTabColorCommand` - Set tab colors
- `GetColorsCommand` - Get current colors

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
Large payloads (>4096 bytes) are automatically split into chunks:

```rust
let message = cmd.build()?;
if message.needs_streaming() {
    let chunks = message.into_chunks();
    for chunk in chunks {
        client.send_raw(&chunk).await?;
    }
}
```

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

## Socket Configuration

kitty-rc communicates with kitty via Unix domain sockets. The default socket location varies by system:

- Linux/macOS: `$TMPDIR/kitty-*` or `/tmp/kitty-*`
- You can also specify custom socket paths

To enable remote control in kitty, add to `kitty.conf`:

```
allow_remote_control yes
listen_on unix:/tmp/kitty-$(pid).sock
```

## Testing

Run the test suite:

```bash
cargo test
```

All 123 tests pass successfully.

## Examples

See `src/bin/list_windows.rs` for a complete example of listing kitty windows and their processes.

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
