# Plan: Enhanced Kitty Focus Tracking with Remote Control Support

## Problem Statement

Currently, `kitty-focus-tracker` has a significant limitation:

1. **Most kitty instances lack remote control setup** - Users may have many long-running kitty terminals that were launched before remote control was configured. Only recently launched kitty instances have the required `listen_on` socket configuration.

2. **No feedback on zooming operations** - When a kitty window gains/loses focus, we attempt to adjust font size, but there's no indication of:
   - Whether the remote control connection succeeded
   - Whether the font size adjustment was applied
   - Which kitty instances actually support zooming
   - Errors from failed remote control operations

3. **Silent failures** - Remote control connection failures are logged to stderr but not reflected in the status output, making it difficult to debug issues.

## Proposed Solution

### 1. Kitty Instance Detection and Categorization

Track kitty windows in categories based on remote control availability:

```rust
enum KittyWindowStatus {
    RemoteControlReady {
        pid: i32,
        socket_path: PathBuf,
    },
    NoRemoteControl {
        pid: i32,
    },
    ConnectionFailed {
        pid: i32,
        error: String,
    },
}
```

**Detection Logic:**

```rust
async fn check_kitty_remote_control(pid: i32) -> KittyWindowStatus {
    let socket_path = get_kitty_socket_path(pid);

    // Try to connect to kitty socket
    // NOTE: Password is NOT sent by client - kitty reads it from its own
    // ~/.config/kitty/rc.conf file. We just need to connect to verify
    // remote control is enabled.

    match timeout(Duration::from_secs(1), UnixStream::connect(&socket_path)).await {
        Ok(_) => {
            // Socket exists - verify kitty accepts commands
            match KittyClient::connect_with_timeout(&socket_path, Duration::from_secs(2)).await {
                Ok(mut client) => {
                    // Try a simple command to verify it works
                    match client.execute(&LsCommand::new().build()?).await {
                        Ok(_) => {
                            client.close().await.ok();
                            KittyWindowStatus::RemoteControlReady { pid, socket_path }
                        }
                        Err(e) => {
                            KittyWindowStatus::ConnectionFailed {
                                pid,
                                error: e.to_string(),
                            }
                        }
                    }
                }
                Err(e) => {
                    KittyWindowStatus::ConnectionFailed {
                        pid,
                        error: e.to_string(),
                    }
                }
            }
        }
        Err(_) => {
            // Socket doesn't exist or no permissions
            KittyWindowStatus::NoRemoteControl { pid }
        }
    }
}
```

### 2. Authentication Clarification

**Important:** `kitty-rc` does **not** require sending passwords. Kitty reads its password from `~/.config/kitty/rc.conf`:
```conf
allow_remote_control password
remote_control_password "$(cat ~/.config/kitty/rc.password)"
```

The client only connects to the socket - no authentication needed at connection time.

### 3. Enhanced Status Output

Add detailed zooming status to JSON output:

```json
{
  "event": "focus_gained",
  "window_id": 12345,
  "app_id": "kitty",
  "zooming": {
    "status": "success",
    "pid": 45678,
    "font_adjustment": "+3",
    "message": "Font size increased by 3 increments"
  }
}
```

**Possible Zooming Status Values:**

- `"success"` - Font size adjustment completed successfully
- `"not_configured"` - Kitty instance lacks remote control setup
- `"connection_failed"` - Could not connect to kitty socket
- `"auth_failed"` - Password authentication failed
- `"not_a_kitty_window"` - Window has wrong PID or doesn't support remote control
- `"no_pid"` - No PID available for the window

**Focus Lost Status:**

```json
{
  "event": "focus_lost",
  "zooming": {
    "status": "partial",
    "affected_kitties": [
      {
        "pid": 45678,
        "status": "success"
      },
      {
        "pid": 45679,
        "status": "not_configured",
        "message": "Remote control not set up"
      }
    ],
    "total": 2,
    "successful": 1,
    "failed": 1
  }
}
```

### 3. Enhanced Command-Line Options

Add options for handling non-configured kitty instances:

```bash
kitty-focus-tracker [OPTIONS]

Options:
  --warn-on-no-remote        Log warning when kitty lacks remote control (default: true)
  --require-remote          Exit if any kitty window lacks remote control
  --setup-remote             Generate remote control setup instructions
  --socket-timeout SECS     Timeout for kitty socket connection (default: 2)
```

**`--setup-remote` Output:**

```bash
kitty-focus-tracker --setup-remote
```

Would display:
```
To enable remote control for kitty:

1. Generate a password:
   pwgen -s 48 1 > ~/.config/kitty/rc.password
   chmod 600 ~/.config/kitty/rc.password

2. Add to ~/.config/kitty/rc.conf:
   allow_remote_control password
   remote_control_password "$(cat ~/.config/kitty/rc.password)"
   listen_on unix:${XDG_RUNTIME_DIR}/kitty/kitty-{kitty_pid}.sock

3. Add to ~/.config/kitty/kitty.conf:
   include rc.conf

4. Restart kitty: The remote control socket will be created at:
   $XDG_RUNTIME_DIR/kitty/kitty-{pid}.sock

Note: Existing kitty windows must be restarted to enable remote control.
```

### 4. Implementation Strategy

**Phase 1: Detection**

1. When a kitty window opens (`WindowOpenedOrChanged`), immediately check remote control status
2. Store status in `HashMap<u64, KittyWindowStatus>`
3. If status is `NoRemoteControl` and `--warn-on-no-remote`, log to stderr
4. If `--require-remote` is set, exit with error code

**Phase 2: Enhanced Focus Events**

When focus changes:

1. **Focus Gained:**
   - Look up window status
   - If `RemoteControlReady`:
     - Attempt font size increase
     - Report success/failure in zooming status
   - If not ready:
     - Report `"status": "not_configured"` in zooming field
   - Output enhanced JSON

2. **Focus Lost:**
   - Iterate through all tracked kitty windows
   - For each window:
     - If `RemoteControlReady`: decrease font size
     - Track individual results
   - Report aggregated results in zooming status

**Phase 3: Error Handling**

- Timeout connections after 2 seconds to avoid blocking
- Catch and report specific error types:
  - `ConnectionRefused` → "connection_failed"
  - `AuthenticationFailed` → "auth_failed"
  - `SocketNotFound` → "not_configured"
- Log detailed errors to stderr
- Include error message in zooming status JSON

### 5. Testing

**Test Scenarios:**

1. **Fresh kitty with remote control:**
   - Should report `"status": "success"`
   - Font size should actually change

2. **Long-running kitty without remote control:**
   - Should report `"status": "not_configured"`
   - Should log warning to stderr
   - Should not attempt connection

3. **Kitty with wrong password:**
   - Should report `"status": "auth_failed"`
   - Should include error message

4. **Multiple kitty instances (mixed):**
   - Focus lost should report `successful` and `failed` counts
   - Each instance status should be detailed in `affected_kitties` array

5. **Non-kitty window with matching app_id:**
   - Should report `"status": "not_a_kitty_window"`
   - Should not attempt connection

### 6. Configuration Recommendations

**For Users:**

- Run `kitty-focus-tracker --setup-remote` once to set up remote control
- Restart long-running kitty instances to enable remote control
- Use `--warn-on-no-remote=false` to suppress warnings
- Use `--require-remote` in automated scripts to ensure all kitty instances are controllable

**For Distribution:**

- Include setup script to automatically configure remote control
- Document that remote control must be configured per kitty instance
- Provide clear error messages with remediation steps

### 7. Backward Compatibility

- Maintain existing JSON output format for basic events
- Add `zooming` field as optional additional detail
- Default behavior: attempt remote control, report errors gracefully
- `--verbose` flag continues to work as before (debug output to stderr)

## Open Questions

1. Should we attempt to auto-configure remote control for unconfigured instances?
   - Pros: Works seamlessly for users
   - Cons: Modifies running processes, security concern

2. Should we cache remote control status or check on each focus event?
   - Cached: Faster, less overhead
   - Per-event: Handles dynamic changes (kitty restarts)

3. What should happen when all kitty windows lack remote control?
   - Option A: Continue tracking, report all as "not_configured"
   - Option B: Exit with error (with `--require-remote`)
   - Option C: Prompt user to configure

## Success Criteria

- [ ] Can detect which kitty windows have remote control
- [ ] Reports detailed zooming status in JSON output
- [ ] Gracefully handles kitty instances without remote control
- [ ] Provides actionable error messages
- [ ] Includes setup instructions
- [ ] Works with mixed remote control configurations
- [ ] Backward compatible with existing JSON format
